use ahbar_shared::{
    is_valid_email, verify_subscriber_token, SignalVote, SubscriptionPreferences,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const MAX_SUBSCRIBERS: usize = 50_000;
const MAX_VOTE_ITEMS: usize = 50_000;

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
struct SubscribeRequest {
    #[serde(flatten)]
    prefs: SubscriptionPreferences,
    #[serde(default)]
    token: Option<String>,
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
                "HTTP/1.1 {}\r\nContent-Type: application/json\r\nX-Content-Type-Options: nosniff\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nContent-Length: {}\r\n\r\n{}",
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
    store_path: &Path,
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
        match serde_json::from_str::<SubscribeRequest>(body) {
            Ok(req) => {
                let mut prefs = req.prefs;
                let email = prefs.email.trim().to_ascii_lowercase();
                if !is_valid_email(&email) {
                    return ("400 Bad Request", r#"{"error":"Invalid email"}"#.to_string());
                }
                prefs.email = email.clone();
                prefs.areas.truncate(10);
                prefs.tags.truncate(30);
                prefs.tags.retain(|t| {
                    !t.is_empty()
                        && t.len() <= 40
                        && t.chars()
                            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
                });
                prefs.lang = if prefs.lang.eq_ignore_ascii_case("en") {
                    "en".to_string()
                } else {
                    "tr".to_string()
                };

                let has_valid_token = req
                    .token
                    .as_deref()
                    .map(|t| verify_subscriber_token(&email, t, secret))
                    .unwrap_or(false);

                if let Ok(mut guard) = state.lock() {
                    if guard.subscribers.contains_key(&email) && !has_valid_token {
                        // Do not allow unauthenticated callers to overwrite an existing subscriber's preferences
                        return (
                            "200 OK",
                            serde_json::json!({
                                "ok": true,
                                "email": email,
                                "requires_magic_link": true,
                                "message": "This address is already subscribed. Use the magic link in your email to modify preferences."
                            })
                            .to_string(),
                        );
                    }
                    if !guard.subscribers.contains_key(&email)
                        && guard.subscribers.len() >= MAX_SUBSCRIBERS
                    {
                        return (
                            "429 Too Many Requests",
                            r#"{"error":"Subscriber capacity reached"}"#.to_string(),
                        );
                    }
                    prefs.verified = has_valid_token;
                    guard.subscribers.insert(email.clone(), prefs.clone());
                    let _ = persist_store(&guard, store_path);
                }
                let resp = serde_json::json!({
                    "ok": true,
                    "email": email,
                    "preferences": prefs
                });
                return ("200 OK", resp.to_string());
            }
            Err(_) => {
                return (
                    "400 Bad Request",
                    r#"{"error":"Invalid subscription payload"}"#.to_string(),
                );
            }
        }
    }

    if path.starts_with("/api/unsubscribe") && (method == "POST" || method == "GET") {
        #[derive(Deserialize)]
        struct UnsubReq {
            email: String,
            token: String,
        }
        let parsed_unsub = if method == "POST" && !body.trim().is_empty() {
            serde_json::from_str::<UnsubReq>(body).ok()
        } else {
            parse_query_unsub(path).map(|(email, token)| UnsubReq { email, token })
        };

        if let Some(req) = parsed_unsub {
            let email = req.email.trim().to_ascii_lowercase();
            if is_valid_email(&email) && verify_subscriber_token(&email, &req.token, secret) {
                if let Ok(mut guard) = state.lock() {
                    guard.subscribers.remove(&email);
                    let _ = persist_store(&guard, store_path);
                }
                return ("200 OK", r#"{"ok":true,"unsubscribed":true}"#.to_string());
            }
            return (
                "403 Forbidden",
                r#"{"error":"Invalid magic link token"}"#.to_string(),
            );
        }
        return (
            "400 Bad Request",
            r#"{"error":"Missing email or token"}"#.to_string(),
        );
    }

    if method == "POST" && path.starts_with("/api/vote") {
        if let Ok(v) = serde_json::from_str::<VoteRequest>(body) {
            let item_id = v.item_id.trim();
            if item_id.is_empty()
                || item_id.len() > 120
                || !item_id
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
            {
                return (
                    "400 Bad Request",
                    r#"{"error":"Invalid item_id"}"#.to_string(),
                );
            }
            if let Ok(mut guard) = state.lock() {
                if !guard.votes.contains_key(item_id) && guard.votes.len() >= MAX_VOTE_ITEMS {
                    return (
                        "429 Too Many Requests",
                        r#"{"error":"Vote store capacity reached"}"#.to_string(),
                    );
                }
                let entry = guard.votes.entry(item_id.to_string()).or_default();
                match v.vote {
                    SignalVote::Signal => entry.signal = entry.signal.saturating_add(1),
                    SignalVote::Noise => entry.noise = entry.noise.saturating_add(1),
                }
                let snapshot = entry.clone();
                let _ = persist_store(&guard, store_path);
                return (
                    "200 OK",
                    serde_json::json!({
                        "ok": true,
                        "item_id": item_id,
                        "signal": snapshot.signal,
                        "noise": snapshot.noise
                    })
                    .to_string(),
                );
            }
        }
        return ("400 Bad Request", r#"{"error":"Invalid vote payload"}"#.to_string());
    }

    ("404 Not Found", r#"{"error":"Not found"}"#.to_string())
}

fn parse_query_unsub(path: &str) -> Option<(String, String)> {
    let query = path.split_once('?')?.1;
    let mut email = None;
    let mut token = None;
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            match k {
                "email" => email = Some(percent_decode(v)),
                "token" => token = Some(percent_decode(v)),
                _ => {}
            }
        }
    }
    Some((email?, token?))
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (
                (bytes[i + 1] as char).to_digit(16),
                (bytes[i + 2] as char).to_digit(16),
            ) {
                out.push(((h << 4) | l) as u8);
                i += 3;
                continue;
            }
        } else if bytes[i] == b'+' {
            out.push(b' ');
            i += 1;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

fn persist_store(store: &StoreData, path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ahbar_shared::sign_subscriber_token;

    #[test]
    fn test_subscribe_does_not_leak_token_or_allow_unauthenticated_overwrite() {
        let state = Arc::new(Mutex::new(StoreData::default()));
        let secret = "unit-test-secret";
        let dummy_path = PathBuf::from("target/test_subscribers_store.json");

        let req1 = concat!(
            "POST /api/subscribe HTTP/1.1\r\n\r\n",
            r#"{"email":"alice@example.org","mode":"standard","density":"three_per_area","frequency":"daily","content_filter":"all","areas":["cs"],"tags":["rust"],"lang":"tr","verified":false}"#
        );
        let (status1, body1) = handle_http_request(req1, &state, secret, &dummy_path);
        assert_eq!(status1, "200 OK");
        assert!(!body1.contains("\"token\""));

        // Attempt unauthenticated overwrite with different areas/tags
        let req2 = concat!(
            "POST /api/subscribe HTTP/1.1\r\n\r\n",
            r#"{"email":"alice@example.org","mode":"personalized","density":"compact","frequency":"weekend_only","content_filter":"papers_only","areas":["econ"],"tags":[],"lang":"en","verified":false}"#
        );
        let (status2, body2) = handle_http_request(req2, &state, secret, &dummy_path);
        assert_eq!(status2, "200 OK");
        assert!(body2.contains("\"requires_magic_link\":true"));

        // Verify original preferences were preserved
        {
            let guard = state.lock().unwrap();
            let saved = guard.subscribers.get("alice@example.org").unwrap();
            assert_eq!(saved.lang, "tr");
            assert_eq!(saved.tags, vec!["rust".to_string()]);
        }

        // Authenticated update using valid magic link token succeeds
        let valid_token = sign_subscriber_token("alice@example.org", secret);
        let req3 = format!(
            "POST /api/subscribe HTTP/1.1\r\n\r\n{{\"email\":\"alice@example.org\",\"token\":\"{}\",\"mode\":\"personalized\",\"density\":\"compact\",\"frequency\":\"daily\",\"content_filter\":\"papers_only\",\"areas\":[\"math\"],\"tags\":[\"logic\"],\"lang\":\"en\",\"verified\":true}}",
            valid_token
        );
        let (status3, _) = handle_http_request(&req3, &state, secret, &dummy_path);
        assert_eq!(status3, "200 OK");
        {
            let guard = state.lock().unwrap();
            let saved = guard.subscribers.get("alice@example.org").unwrap();
            assert_eq!(saved.lang, "en");
            assert_eq!(saved.tags, vec!["logic".to_string()]);
            assert!(saved.verified);
        }

        let _ = fs::remove_file(&dummy_path);
    }
}
