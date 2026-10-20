mod emit;
mod ingest;
mod mailer;
mod seed;
mod triage;

use ahbar_shared::{
    Area, ContentFilter, DailyIssue, DigestDensity, DigestFrequency, DigestMode, EditionType,
    IssueStats, SourceHealthRecord, SubscriptionPreferences, SOURCES,
};
use chrono::{Datelike, NaiveDate, Utc, Weekday};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let offline_mode = args.iter().any(|a| a == "--offline");
    let root_dir = args
        .windows(2)
        .find(|w| w[0] == "--root")
        .map(|w| PathBuf::from(&w[1]))
        .unwrap_or(env::current_dir()?);

    let now = Utc::now();
    let date_str = args
        .windows(2)
        .find(|w| w[0] == "--date")
        .map(|w| w[1].clone())
        .unwrap_or_else(|| now.format("%Y-%m-%d").to_string());

    let parsed_date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
        .unwrap_or_else(|_| now.date_naive());
    let edition_type = match parsed_date.weekday() {
        Weekday::Sat | Weekday::Sun => EditionType::Weekend,
        _ => EditionType::Daily,
    };

    let site_base_url = env::var("AHBAR_BASE_URL")
        .unwrap_or_else(|_| "https://latifkedi.github.io/ahbar".to_string());
    let hmac_secret = env::var("AHBAR_HMAC_SECRET")
        .unwrap_or_else(|_| "ahbar-default-dev-hmac-secret-key".to_string());

    println!(
        "[Ahbar Engine] Building {:?} Issue for {} (offline={})",
        edition_type, date_str, offline_mode
    );

    let now_iso = now.to_rfc3339();
    let mut raw_candidates = Vec::new();
    let mut health_records = Vec::new();

    if !offline_mode {
        let fetched = ingest::fetch_all_sources().await;
        raw_candidates = fetched.candidates;
        health_records = fetched.health;
    } else {
        for src in SOURCES {
            health_records.push(SourceHealthRecord {
                id: src.id.to_string(),
                name: src.name.to_string(),
                area: src.area,
                url: src.url.to_string(),
                status: "offline_seed".to_string(),
                items_fetched: 3,
                latency_ms: 0,
                last_checked: now_iso.clone(),
                error: None,
            });
        }
    }

    // Ensure every one of the 7 areas has at least 3 high-rigor dispatches
    let baseline = seed::baseline_candidates(&now_iso);
    let mut area_counts: HashMap<Area, usize> = HashMap::new();
    for c in &raw_candidates {
        *area_counts.entry(c.area).or_insert(0) += 1;
    }
    for base_cand in baseline {
        if *area_counts.get(&base_cand.area).unwrap_or(&0) < 3 {
            *area_counts.entry(base_cand.area).or_insert(0) += 1;
            raw_candidates.push(base_cand);
        }
    }

    let raw_items_scanned = raw_candidates.len();
    let clusters = triage::stage1_cluster_and_rank(raw_candidates);
    let clusters_formed = clusters.len();

    let next_issue_number = compute_issue_number(&root_dir, &date_str);
    let items =
        triage::stage2_evaluate_and_summarize(clusters, &date_str, next_issue_number).await;

    let top_picks_count = items.iter().filter(|i| i.is_top_pick).count();
    let sources_healthy = health_records
        .iter()
        .filter(|h| h.status == "ok" || h.status == "offline_seed")
        .count();

    let issue = DailyIssue {
        issue_number: next_issue_number,
        date: date_str.clone(),
        edition_type,
        generated_at: now_iso,
        stats: IssueStats {
            sources_checked: SOURCES.len(),
            sources_healthy,
            raw_items_scanned,
            clusters_formed,
            qualified_items: items.len(),
            top_picks_count,
        },
        items,
    };

    emit::emit_all(&root_dir, &issue, &health_records, &site_base_url)?;

    // Render sample email previews (10 areas × 3 items = 30 items) in both English and Turkish
    let default_prefs = SubscriptionPreferences {
        email: "reader@example.org".to_string(),
        mode: DigestMode::Standard,
        density: DigestDensity::ThreePerArea,
        frequency: DigestFrequency::Daily,
        content_filter: ContentFilter::All,
        areas: Area::ALL.to_vec(),
        tags: Vec::new(),
        lang: "en".to_string(),
        verified: true,
    };
    let selected_for_preview = mailer::select_items_for_subscriber(&issue, &default_prefs);
    let preview_html_en = mailer::render_digest_email_html_with_lang(
        &issue,
        &selected_for_preview,
        &default_prefs.email,
        &site_base_url,
        &hmac_secret,
        "en",
    );
    let preview_html_tr = mailer::render_digest_email_html_with_lang(
        &issue,
        &selected_for_preview,
        &default_prefs.email,
        &site_base_url,
        &hmac_secret,
        "tr",
    );
    fs::write(root_dir.join("data").join("sample-email.html"), &preview_html_en)?;
    fs::write(root_dir.join("data").join("sample-email-tr.html"), &preview_html_tr)?;

    if args.iter().any(|a| a == "--send-emails") {
        let client = reqwest::Client::new();
        let default_lang = env::var("AHBAR_EMAIL_LANG").unwrap_or_else(|_| "tr".to_string());
        let subscribers = collect_active_subscribers(&client, &root_dir, &default_lang).await;

        if subscribers.is_empty() {
            println!("[Ahbar Engine] Notice: --send-emails specified but no active subscribers found (check AHBAR_SUBSCRIBERS_URL, RESEND_AUDIENCE_ID, data/subscribers.json, or AHBAR_TO_EMAIL)");
        } else {
            println!(
                "[Ahbar Engine] Starting automated daily dispatch to {} subscriber(s)...",
                subscribers.len()
            );

            for sub in &subscribers {
                let selected = mailer::select_items_for_subscriber(&issue, sub);
                if selected.is_empty() {
                    continue;
                }
                let html = mailer::render_digest_email_html_with_lang(
                    &issue,
                    &selected,
                    &sub.email,
                    &site_base_url,
                    &hmac_secret,
                    &sub.lang,
                );
                let subject = if sub.lang == "tr" {
                    format!("Ahbar Sayı #{} — {}", issue.issue_number, issue.date)
                } else {
                    format!("Ahbar Issue #{} — {}", issue.issue_number, issue.date)
                };

                match mailer::dispatch_email_via_provider(
                    &client,
                    &sub.email,
                    &subject,
                    &html,
                )
                .await
                {
                    Ok(provider) => println!(
                        "[Ahbar Engine] Successfully delivered issue to {} via {}",
                        sub.email, provider
                    ),
                    Err(e) => eprintln!(
                        "[Ahbar Engine] Delivery failed for {}: {}",
                        sub.email, e
                    ),
                }
            }
        }
    }

    if args.iter().any(|a| a == "--send-telegram") || env::var("TELEGRAM_BOT_TOKEN").is_ok() {
        if let (Ok(token), Ok(chat_id)) =
            (env::var("TELEGRAM_BOT_TOKEN"), env::var("TELEGRAM_CHAT_ID"))
        {
            let client = reqwest::Client::new();
            match mailer::dispatch_telegram_digest(
                &client,
                &issue,
                &token,
                &chat_id,
                &site_base_url,
            )
            .await
            {
                Ok(_) => println!(
                    "[Ahbar Engine] Dispatched Telegram daily digest to {}",
                    chat_id
                ),
                Err(e) => eprintln!("[Ahbar Engine] Telegram dispatch error: {}", e),
            }
        }
    }

    println!(
        "[Ahbar Engine] Emitted Issue #{} ({}) -> {} qualified items ({} Top Picks, {} in 10x3 email)",
        issue.issue_number,
        issue.date,
        issue.stats.qualified_items,
        issue.stats.top_picks_count,
        selected_for_preview.len()
    );

    Ok(())
}


async fn collect_active_subscribers(
    client: &reqwest::Client,
    root_dir: &Path,
    default_lang: &str,
) -> Vec<SubscriptionPreferences> {
    let mut subscribers: Vec<SubscriptionPreferences> = Vec::new();
    let mut seen_emails = HashSet::new();

    // 1. From local data/subscribers.json (if present)
    let subs_file = root_dir.join("data").join("subscribers.json");
    if subs_file.exists() {
        if let Ok(raw) = fs::read_to_string(&subs_file) {
            if let Ok(list) = serde_json::from_str::<Vec<SubscriptionPreferences>>(&raw) {
                for s in list {
                    if s.verified && seen_emails.insert(s.email.to_lowercase()) {
                        subscribers.push(s);
                    }
                }
            }
        }
    }

    // 2. From Cloudflare Worker API (AHBAR_SUBSCRIBERS_URL)
    if let Ok(worker_url) = env::var("AHBAR_SUBSCRIBERS_URL") {
        let secret = env::var("AHBAR_HMAC_SECRET").unwrap_or_else(|_| "ahbar-default-dev-hmac-secret-key".to_string());
        if let Ok(resp) = client.get(&worker_url).bearer_auth(&secret).send().await {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(list) = json.get("subscribers").and_then(|v| v.as_array()) {
                        for item in list {
                            if let Some(email) = item.get("email").and_then(|v| v.as_str()) {
                                let email = email.trim().to_lowercase();
                                if !email.is_empty() && seen_emails.insert(email.clone()) {
                                    let lang = item.get("lang").and_then(|v| v.as_str()).unwrap_or(default_lang).to_string();
                                    subscribers.push(SubscriptionPreferences {
                                        email,
                                        mode: DigestMode::Standard,
                                        density: DigestDensity::ThreePerArea,
                                        frequency: DigestFrequency::Daily,
                                        content_filter: ContentFilter::All,
                                        areas: Area::ALL.to_vec(),
                                        tags: Vec::new(),
                                        lang,
                                        verified: true,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. From Resend Audience API (if RESEND_AUDIENCE_ID and RESEND_API_KEY are configured)
    if let (Ok(resend_key), Ok(aud_id)) = (env::var("RESEND_API_KEY"), env::var("RESEND_AUDIENCE_ID")) {
        let url = format!("https://api.resend.com/audiences/{}/contacts", aud_id);
        if let Ok(resp) = client.get(&url).bearer_auth(&resend_key).send().await {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(contacts) = json.get("data").and_then(|v| v.as_array()) {
                        for c in contacts {
                            let unsub = c.get("unsubscribed").and_then(|v| v.as_bool()).unwrap_or(false);
                            if !unsub {
                                if let Some(em) = c.get("email").and_then(|v| v.as_str()) {
                                    let em = em.trim().to_lowercase();
                                    if !em.is_empty() && seen_emails.insert(em.clone()) {
                                        subscribers.push(SubscriptionPreferences {
                                            email: em,
                                            mode: DigestMode::Standard,
                                            density: DigestDensity::ThreePerArea,
                                            frequency: DigestFrequency::Daily,
                                            content_filter: ContentFilter::All,
                                            areas: Area::ALL.to_vec(),
                                            tags: Vec::new(),
                                            lang: default_lang.to_string(),
                                            verified: true,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 4. From AHBAR_TO_EMAIL environment fallback
    if let Ok(direct_emails) = env::var("AHBAR_TO_EMAIL") {
        for em in direct_emails.split(',').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()) {
            if seen_emails.insert(em.clone()) {
                subscribers.push(SubscriptionPreferences {
                    email: em,
                    mode: DigestMode::Standard,
                    density: DigestDensity::ThreePerArea,
                    frequency: DigestFrequency::Daily,
                    content_filter: ContentFilter::All,
                    areas: Area::ALL.to_vec(),
                    tags: Vec::new(),
                    lang: default_lang.to_string(),
                    verified: true,
                });
            }
        }
    }

    subscribers
}

fn compute_issue_number(root_dir: &Path, date_str: &str) -> u32 {
    let archive_path = root_dir.join("data").join("archive.json");
    if let Ok(raw) = fs::read_to_string(archive_path) {
        if let Ok(entries) = serde_json::from_str::<Vec<ahbar_shared::ArchiveEntry>>(&raw) {
            if let Some(existing) = entries.iter().find(|e| e.date == date_str) {
                return existing.issue_number;
            }
            return entries.iter().map(|e| e.issue_number).max().unwrap_or(0) + 1;
        }
    }
    1
}
