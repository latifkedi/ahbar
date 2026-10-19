use ahbar_shared::{
    sign_subscriber_token, verify_subscriber_token, SignalVote, SubscriptionPreferences,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[derive(Debug, Default, Serialize, Deserialize)]
struct StoreData {
    subscribers: HashMap<String, SubscriptionPreferences>,
    votes: HashMap<String, ItemVoteTally>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct ItemVoteTally {
    signal: u32,
    noise: u32,
}

#[derive(Deserialize)]
struct VoteRequest {
    item_id: String,
    vote: SignalVote,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bind_addr = env::var("AHBAR_API_ADDR").unwrap_or_else(|_| "127.0.0.1:8787".to_string());
    let secret = env::var("AHBAR_HMAC_SECRET")
        .unwrap_or_else(|_| "ahbar-default-dev-hmac-secret-key".to_string());
    let store_path = PathBuf::from("data/subscribers_store.json");

    let initial_store = if store_path.exists() {
        let raw = fs::read_to_string(&store_path).unwrap_or_default();
        serde_json::from_str(&raw).unwrap_or_default()
    } else {
        StoreData::default()
    };

    let state = Arc::new(Mutex::new(initial_store));
    let listener = TcpListener::bind(&bind_addr).await?;
    println!("[Ahbar API] Listening on http://{}", bind_addr);

    loop {
        let (mut socket, _) = listener.accept().await?;
        let state_clone = Arc::clone(&state);
        let secret_clone = secret.clone();
        let store_path_clone = store_path.clone();

        tokio::spawn(async move {
            let mut buf = vec![0u8; 16384];
            let n = match socket.read(&mut buf).await {
                Ok(n) if n > 0 => n,
                _ => return,
            };
            let req_str = String::from_utf8_lossy(&buf[..n]);
            let (status, body) =
                handle_http_request(&req_str, &state_clone, &secret_clone, &store_path_clone);

            let response = format!(
                "HTTP/1.1 {}\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nContent-Length: {}\r\n\r\n{}",
                status,
                body.len(),
                body
            );
            let _ = socket.write_all(response.as_bytes()).await;
        });
    }
}

fn handle_http_request(
    raw_req: &str,
    state: &Arc<Mutex<StoreData>>,
    secret: &str,
    store_path: &PathBuf,
) -> (&'static str, String) {
    let mut lines = raw_req.lines();
    let first_line = lines.next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return ("400 Bad Request", r#"{"error":"Malformed request"}"#.to_string());
    }
    let method = parts[0];
    let path = parts[1];

    if method == "OPTIONS" {
        return ("200 OK", r#"{"ok":true}"#.to_string());
    }

    let body = raw_req
        .split("\r\n\r\n")
        .nth(1)
        .unwrap_or("")
        .trim_matches(char::from(0));

    if method == "POST" && path.starts_with("/api/subscribe") {
        match serde_json::from_str::<SubscriptionPreferences>(body) {
            Ok(mut prefs) => {
                let email = prefs.email.trim().to_ascii_lowercase();
                if !email.contains('@') {
                    return ("400 Bad Request", r#"{"error":"Invalid email"}"#.to_string());
                }
                prefs.email = email.clone();
                prefs.verified = true;
                let token = sign_subscriber_token(&email, secret);
                if let Ok(mut guard) = state.lock() {
                    guard.subscribers.insert(email.clone(), prefs.clone());
                    let _ = persist_store(&guard, store_path);
                }
                let resp = serde_json::json!({
                    "ok": true,
                    "email": email,
                    "token": token,
                    "preferences": prefs
                });
                return ("200 OK", resp.to_string());
            }
            Err(e) => {
                return (
                    "400 Bad Request",
                    serde_json::json!({ "error": e.to_string() }).to_string(),
                );
            }
        }
    }

    if method == "POST" && path.starts_with("/api/unsubscribe") {
        #[derive(Deserialize)]
        struct UnsubReq {
            email: String,
            token: String,
        }
        if let Ok(req) = serde_json::from_str::<UnsubReq>(body) {
            if verify_subscriber_token(&req.email, &req.token, secret) {
                if let Ok(mut guard) = state.lock() {
                    guard.subscribers.remove(&req.email.to_ascii_lowercase());
                    let _ = persist_store(&guard, store_path);
                }
                return ("200 OK", r#"{"ok":true,"unsubscribed":true}"#.to_string());
            }
            return ("403 Forbidden", r#"{"error":"Invalid magic link token"}"#.to_string());
        }
    }

    if method == "POST" && path.starts_with("/api/vote") {
        if let Ok(v) = serde_json::from_str::<VoteRequest>(body) {
            if let Ok(mut guard) = state.lock() {
                let entry = guard.votes.entry(v.item_id.clone()).or_default();
                match v.vote {
                    SignalVote::Signal => entry.signal += 1,
                    SignalVote::Noise => entry.noise += 1,
                }
                let snapshot = entry.clone();
                let _ = persist_store(&guard, store_path);
                return (
                    "200 OK",
                    serde_json::json!({
                        "ok": true,
                        "item_id": v.item_id,
                        "signal": snapshot.signal,
                        "noise": snapshot.noise
                    })
                    .to_string(),
                );
            }
        }
    }

    ("404 Not Found", r#"{"error":"Not found"}"#.to_string())
}

fn persist_store(store: &StoreData, path: &PathBuf) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}
