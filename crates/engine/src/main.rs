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
            let force_resend = args.iter().any(|a| a == "--force-send-emails" || a == "--force-resend");
            let ledger_path = root_dir.join("data").join("dispatch-log.json");
            let mut ledger = load_dispatch_ledger(&ledger_path);
            let today_sent = ledger.entry(issue.date.clone()).or_default();
            let mut dispatched_count = 0;

            println!(
                "[Ahbar Engine] Starting automated daily dispatch to {} unique subscriber(s)...",
                subscribers.len()
            );

            for sub in &subscribers {
                let norm_email = normalize_email(&sub.email);
                if !force_resend && today_sent.contains(&norm_email) {
                    println!(
                        "[Ahbar Engine] Skipping '{}': already received today's issue ({}). Use --force-resend to override.",
                        norm_email, issue.date
                    );
                    continue;
                }

                let selected = mailer::select_items_for_subscriber(&issue, sub);
                if selected.is_empty() {
                    continue;
                }
                let html = mailer::render_digest_email_html_with_lang(
                    &issue,
                    &selected,
                    &norm_email,
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
                    &norm_email,
                    &subject,
                    &html,
                )
                .await
                {
                    Ok(provider) => {
                        println!(
                            "[Ahbar Engine] Successfully delivered issue to {} via {}",
                            norm_email, provider
                        );
                        today_sent.insert(norm_email);
                        dispatched_count += 1;
                    }
                    Err(e) => eprintln!(
                        "[Ahbar Engine] Delivery failed for {}: {}",
                        norm_email, e
                    ),
                }
            }

            if dispatched_count > 0 {
                save_dispatch_ledger(&ledger_path, &ledger);
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


#[inline]
fn normalize_email(raw: &str) -> String {
    raw.trim().to_lowercase()
}

fn load_dispatch_ledger(path: &Path) -> HashMap<String, HashSet<String>> {
    if path.exists() {
        if let Ok(raw) = fs::read_to_string(path) {
            if let Ok(map) = serde_json::from_str::<HashMap<String, HashSet<String>>>(&raw) {
                return map;
            }
        }
    }
    HashMap::new()
}

fn save_dispatch_ledger(path: &Path, ledger: &HashMap<String, HashSet<String>>) {
    // Write atomically / safely
    if let Ok(json) = serde_json::to_string_pretty(ledger) {
        let _ = fs::write(path, json);
    }
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
                for mut s in list {
                    let norm = normalize_email(&s.email);
                    if !norm.is_empty() && s.verified && seen_emails.insert(norm.clone()) {
                        s.email = norm;
                        subscribers.push(s);
                    }
                }
            }
        }
    }

    // 2. From Google Sheets published CSV (AHBAR_SHEET_CSV_URL)
    if let Ok(sheet_url) = env::var("AHBAR_SHEET_CSV_URL") {
        let trimmed = sheet_url.trim();
        if !trimmed.is_empty() {
            if let Ok(resp) = client.get(trimmed).send().await {
                if resp.status().is_success() {
                    if let Ok(csv_text) = resp.text().await {
                        parse_subscribers_from_csv(&csv_text, default_lang, &mut seen_emails, &mut subscribers);
                    }
                }
            }
        }
    }

    // 3. From Cloudflare Worker API (AHBAR_SUBSCRIBERS_URL)
    if let Ok(worker_url) = env::var("AHBAR_SUBSCRIBERS_URL") {
        let secret = env::var("AHBAR_HMAC_SECRET").unwrap_or_else(|_| "ahbar-default-dev-hmac-secret-key".to_string());
        if let Ok(resp) = client.get(&worker_url).bearer_auth(&secret).send().await {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(list) = json.get("subscribers").and_then(|v| v.as_array()) {
                        for item in list {
                            if let Some(email) = item.get("email").and_then(|v| v.as_str()) {
                                let norm = normalize_email(email);
                                if !norm.is_empty() && seen_emails.insert(norm.clone()) {
                                    let lang = item.get("lang").and_then(|v| v.as_str()).unwrap_or(default_lang).to_string();
                                    subscribers.push(SubscriptionPreferences {
                                        email: norm,
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
                                    let norm = normalize_email(em);
                                    if !norm.is_empty() && seen_emails.insert(norm.clone()) {
                                        subscribers.push(SubscriptionPreferences {
                                            email: norm,
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

    // 4. From Brevo Contacts API (if BREVO_API_KEY is configured)
    if let Ok(brevo_key) = env::var("BREVO_API_KEY") {
        if !brevo_key.is_empty() {
            let url = if let Ok(list_id) = env::var("BREVO_LIST_ID") {
                format!("https://api.brevo.com/v3/contacts/lists/{}/contacts?limit=500", list_id)
            } else {
                "https://api.brevo.com/v3/contacts?limit=500".to_string()
            };
            if let Ok(resp) = client.get(&url).header("api-key", &brevo_key).send().await {
                if resp.status().is_success() {
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        if let Some(contacts) = json.get("contacts").and_then(|v| v.as_array()) {
                            for c in contacts {
                                let blacklisted = c.get("emailBlacklisted").and_then(|v| v.as_bool()).unwrap_or(false);
                                if !blacklisted {
                                    if let Some(em) = c.get("email").and_then(|v| v.as_str()) {
                                        let norm = normalize_email(em);
                                        if !norm.is_empty() && seen_emails.insert(norm.clone()) {
                                            subscribers.push(SubscriptionPreferences {
                                                email: norm,
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
    }

    // 5. From AHBAR_TO_EMAIL environment fallback
    if let Ok(direct_emails) = env::var("AHBAR_TO_EMAIL") {
        for em in direct_emails.split(',').map(normalize_email).filter(|s| !s.is_empty()) {
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

fn parse_subscribers_from_csv(
    csv_text: &str,
    default_lang: &str,
    seen_emails: &mut HashSet<String>,
    subscribers: &mut Vec<SubscriptionPreferences>,
) {
    for line in csv_text.lines().skip(1) {
        let cols: Vec<&str> = line
            .split(',')
            .map(|s| s.trim().trim_matches('"').trim())
            .collect();
        let email_candidate = cols
            .iter()
            .find(|col| col.contains('@') && !col.starts_with("http"));
        if let Some(em) = email_candidate {
            let norm = normalize_email(em);
            if !norm.is_empty() && seen_emails.insert(norm.clone()) {
                let lang_col = cols
                    .iter()
                    .find(|col| col.eq_ignore_ascii_case("tr") || col.eq_ignore_ascii_case("en"));
                let sub_lang = lang_col
                    .map(|l| l.to_lowercase())
                    .unwrap_or_else(|| default_lang.to_string());
                subscribers.push(SubscriptionPreferences {
                    email: norm,
                    mode: DigestMode::Standard,
                    density: DigestDensity::ThreePerArea,
                    frequency: DigestFrequency::Daily,
                    content_filter: ContentFilter::All,
                    areas: Area::ALL.to_vec(),
                    tags: Vec::new(),
                    lang: sub_lang,
                    verified: true,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_google_sheets_csv() {
        let sample_csv = "Zaman damgası,E-posta,Bülten Dili\n\
                          \"2026/10/20 18:30:00\",alice@example.com,tr\n\
                          \"2026/10/20 18:35:00\",Bob@Example.Com,EN\n\
                          \"2026/10/20 18:40:00\",ALICE@example.com,tr\n"; // duplicate
        let mut seen = HashSet::new();
        let mut subs = Vec::new();
        parse_subscribers_from_csv(sample_csv, "tr", &mut seen, &mut subs);

        assert_eq!(subs.len(), 2);
        assert_eq!(subs[0].email, "alice@example.com");
        assert_eq!(subs[0].lang, "tr");
        assert_eq!(subs[1].email, "bob@example.com");
        assert_eq!(subs[1].lang, "en");
    }

    #[test]
    fn test_normalize_email_trims_and_lowercases() {
        assert_eq!(normalize_email("  User@Example.COM  "), "user@example.com");
        assert_eq!(normalize_email("test.email+tag@gmail.com"), "test.email+tag@gmail.com");
    }

    #[test]
    fn test_dispatch_ledger_prevents_duplicate_sends() {
        let temp_dir = std::env::temp_dir().join(format!("ahbar-test-ledger-{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);
        let ledger_path = temp_dir.join("dispatch-log.json");

        let mut ledger = load_dispatch_ledger(&ledger_path);
        let today = "2026-10-20".to_string();
        let email = "reader@example.org".to_string();

        let today_sent = ledger.entry(today.clone()).or_default();
        assert!(!today_sent.contains(&email));

        today_sent.insert(email.clone());
        save_dispatch_ledger(&ledger_path, &ledger);

        // Reload and verify persistence
        let reloaded = load_dispatch_ledger(&ledger_path);
        assert!(reloaded.get(&today).unwrap().contains(&email));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
