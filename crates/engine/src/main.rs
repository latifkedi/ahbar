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
use std::collections::HashMap;
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
        if let Ok(target_emails) = env::var("AHBAR_TO_EMAIL") {
            let client = reqwest::Client::new();
            let subject = format!("Ahbar Sayı #{} — {}", issue.issue_number, issue.date);
            let email_html = if env::var("AHBAR_EMAIL_LANG").unwrap_or_else(|_| "tr".to_string()) == "tr" {
                &preview_html_tr
            } else {
                &preview_html_en
            };
            for target_email in target_emails.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                println!("[Ahbar Engine] Dispatching daily email digest to {}...", target_email);
                match mailer::dispatch_email_via_provider(
                    &client,
                    target_email,
                    &subject,
                    email_html,
                )
                .await
                {
                    Ok(provider) => println!("[Ahbar Engine] Dispatched email digest to {} via {}", target_email, provider),
                    Err(e) => eprintln!("[Ahbar Engine] Failed to dispatch email to {}: {}", target_email, e),
                }
            }
        } else {
            println!("[Ahbar Engine] Notice: --send-emails specified but AHBAR_TO_EMAIL is not set in environment");
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
