use crate::emit::escape_xml;
use ahbar_shared::{
    sign_subscriber_token, ContentFilter, DailyIssue, DigestFrequency, DigestMode, EditionType,
    NewsItem, SubscriptionPreferences,
};
use reqwest::Client;
use std::env;

/// Selects the exact list of items for a subscriber according to their
/// Mode (Standard / Personalized / Hybrid), Density (Compact 5-7 / Comprehensive 15-20),
/// Frequency, and ContentFilter (All / PapersOnly / CodeOnly).
pub fn select_items_for_subscriber<'a>(
    issue: &'a DailyIssue,
    prefs: &SubscriptionPreferences,
) -> Vec<&'a NewsItem> {
    // Check frequency compatibility
    match (issue.edition_type, prefs.frequency) {
        (EditionType::Weekend, DigestFrequency::WeekdaysOnly) => return Vec::new(),
        (EditionType::Daily, DigestFrequency::WeekendOnly) => return Vec::new(),
        _ => {}
    }

    let max_items = prefs.density.max_items();

    let matches_content_filter = |item: &NewsItem| -> bool {
        match prefs.content_filter {
            ContentFilter::All => true,
            ContentFilter::PapersOnly => item.is_paper,
            ContentFilter::CodeOnly => item.links.code_url.is_some(),
        }
    };

    let matches_personal_interest = |item: &NewsItem| -> bool {
        let area_match = prefs.areas.is_empty() || prefs.areas.contains(&item.area);
        let tag_match = prefs.tags.is_empty()
            || item
                .tags
                .iter()
                .any(|t| prefs.tags.iter().any(|pt| pt == t));
        if !prefs.areas.is_empty() && !prefs.tags.is_empty() {
            area_match || tag_match
        } else if !prefs.areas.is_empty() {
            area_match
        } else if !prefs.tags.is_empty() {
            tag_match
        } else {
            true
        }
    };

    let mut selected: Vec<&'a NewsItem> = Vec::new();

    match prefs.mode {
        DigestMode::Standard => {
            for item in issue.items.iter().filter(|i| i.is_top_pick) {
                if matches_content_filter(item) {
                    selected.push(item);
                    if selected.len() >= max_items {
                        break;
                    }
                }
            }
            if selected.len() < max_items {
                for item in &issue.items {
                    if !selected.iter().any(|s| s.id == item.id) && matches_content_filter(item) {
                        selected.push(item);
                        if selected.len() >= max_items {
                            break;
                        }
                    }
                }
            }
        }
        DigestMode::Personalized => {
            for item in &issue.items {
                if matches_content_filter(item) && matches_personal_interest(item) {
                    selected.push(item);
                    if selected.len() >= max_items {
                        break;
                    }
                }
            }
        }
        DigestMode::Hybrid => {
            // Half top picks, half personalized matches
            let half = (max_items / 2).max(3);
            for item in issue.items.iter().filter(|i| i.is_top_pick) {
                if matches_content_filter(item) {
                    selected.push(item);
                    if selected.len() >= half {
                        break;
                    }
                }
            }
            for item in &issue.items {
                if !selected.iter().any(|s| s.id == item.id)
                    && matches_content_filter(item)
                    && matches_personal_interest(item)
                {
                    selected.push(item);
                    if selected.len() >= max_items {
                        break;
                    }
                }
            }
        }
    }

    selected
}

/// Renders the minimalist editorial HTML email (starts directly with Item #1, zero intro fluff).
/// Clicking any headline takes the reader to the Item Detail Page (`?item=<id>`) on Ahbar.
pub fn render_digest_email_html(
    issue: &DailyIssue,
    items: &[&NewsItem],
    subscriber_email: &str,
    site_base_url: &str,
    hmac_secret: &str,
) -> String {
    let base = site_base_url.trim_end_matches('/');
    let token = sign_subscriber_token(subscriber_email, hmac_secret);
    let prefs_url = format!(
        "{}/?prefs=1&email={}&token={}",
        base,
        urlencoding_simple(subscriber_email),
        token
    );
    let unsub_url = format!(
        "{}/?unsub=1&email={}&token={}",
        base,
        urlencoding_simple(subscriber_email),
        token
    );

    let edition_badge = match issue.edition_type {
        EditionType::Daily => format!("ISSUE #{} · {}", issue.issue_number, issue.date),
        EditionType::Weekend => format!("WEEKEND EDITION #{} · {}", issue.issue_number, issue.date),
    };

    let mut html = String::with_capacity(16384);
    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\n");
    html.push_str(&format!(
        "<title>Ahbar · {}</title>\n</head>\n",
        escape_xml(&edition_badge)
    ));
    html.push_str("<body style=\"margin:0;padding:24px 14px;background-color:#f6f1e7;color:#1c1a16;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;line-height:1.6;\">\n");
    html.push_str("<table role=\"presentation\" width=\"100%\" cellpadding=\"0\" cellspacing=\"0\" style=\"max-width:680px;margin:0 auto;background-color:#fbf8f1;border:1px solid #ddd5c4;border-radius:6px;padding:24px 28px;\">\n");

    // Minimal header bar — goes straight into Item #1 with zero intro paragraph
    html.push_str("<tr><td style=\"padding-bottom:14px;border-bottom:1px solid #ddd5c4;\">\n");
    html.push_str("<table role=\"presentation\" width=\"100%\" cellpadding=\"0\" cellspacing=\"0\"><tr>\n");
    html.push_str(&format!(
        "<td><a href=\"{}/\" style=\"font-family:Georgia,'Times New Roman',serif;font-size:21px;font-weight:700;color:#1c1a16;text-decoration:none;letter-spacing:-0.01em;\"><span style=\"color:#a8321f;\">●</span> Ahbar</a></td>\n",
        escape_xml(base)
    ));
    html.push_str(&format!(
        "<td align=\"right\" style=\"font-size:12px;font-weight:600;letter-spacing:0.1em;color:#a8321f;text-transform:uppercase;\">{}</td>\n",
        escape_xml(&edition_badge)
    ));
    html.push_str("</tr></table>\n</td></tr>\n");

    // Items list (starting directly with #1)
    for (idx, item) in items.iter().enumerate() {
        let num = idx + 1;
        let detail_url = format!("{}/?item={}", base, item.id);
        let vote_sig_url = format!("{}/?vote=signal&item={}", base, item.id);
        let vote_noise_url = format!("{}/?vote=noise&item={}", base, item.id);

        html.push_str("<tr><td style=\"padding:18px 0;border-bottom:1px solid #e9e2d3;\">\n");

        // Area & Source row
        html.push_str(&format!(
            "<div style=\"font-size:11.5px;color:#686256;margin-bottom:5px;\"><strong style=\"color:#a8321f;font-family:Georgia,serif;\">{:02}.</strong> <span style=\"text-transform:uppercase;letter-spacing:0.08em;font-weight:600;color:#5b564c;\">{}</span> · <span>{}</span></div>\n",
            num,
            escape_xml(item.area.title()),
            escape_xml(&item.source_name)
        ));

        // 1-2 Sentence Technical Headline linking to Ahbar Item Detail Page
        html.push_str(&format!(
            "<div style=\"margin:0 0 8px 0;\"><a href=\"{}\" style=\"font-family:Georgia,'Times New Roman',serif;font-size:17.5px;font-weight:600;line-height:1.35;color:#1c1a16;text-decoration:none;\">{}</a></div>\n",
            escape_xml(&detail_url),
            escape_xml(&item.headline)
        ));

        // Single-paragraph Technical Summary
        html.push_str(&format!(
            "<p style=\"margin:0 0 10px 0;font-size:14.5px;line-height:1.58;color:#1c1a16;\">{}</p>\n",
            escape_xml(&item.summary)
        ));

        // Direct Badges + Canonical Tags + Signal/Noise links
        html.push_str("<div style=\"font-size:12px;color:#5b564c;\">\n");
        html.push_str(&format!(
            "<a href=\"{}\" style=\"color:#a8321f;font-weight:600;text-decoration:none;margin-right:10px;\">[Analysis &amp; Details]</a>",
            escape_xml(&detail_url)
        ));
        html.push_str(&format!(
            "<a href=\"{}\" style=\"color:#5b564c;text-decoration:underline;margin-right:10px;\">[Source ↗]</a>",
            escape_xml(&item.url)
        ));
        if let Some(ref pdf) = item.links.pdf_url {
            html.push_str(&format!(
                "<a href=\"{}\" style=\"color:#5b564c;text-decoration:underline;margin-right:10px;\">[PDF]</a>",
                escape_xml(pdf)
            ));
        }
        if let Some(ref ar5iv) = item.links.html_ar5iv_url {
            html.push_str(&format!(
                "<a href=\"{}\" style=\"color:#5b564c;text-decoration:underline;margin-right:10px;\">[HTML / ar5iv]</a>",
                escape_xml(ar5iv)
            ));
        }
        if let Some(ref code) = item.links.code_url {
            html.push_str(&format!(
                "<a href=\"{}\" style=\"color:#5b564c;text-decoration:underline;margin-right:10px;\">[Code]</a>",
                escape_xml(code)
            ));
        }
        for tag in &item.tags {
            html.push_str(&format!(
                "<span style=\"display:inline-block;background-color:#efe8da;border:1px solid #ddd5c4;border-radius:999px;padding:1px 8px;margin-right:5px;font-size:11.5px;color:#5b564c;\">#{}</span>",
                escape_xml(tag)
            ));
        }
        html.push_str(&format!(
            "<span style=\"float:right;\"><a href=\"{}\" style=\"color:#686256;text-decoration:none;margin-right:6px;\" title=\"High signal\">▲ Signal</a><a href=\"{}\" style=\"color:#686256;text-decoration:none;\" title=\"Noise\">▼ Noise</a></span>",
            escape_xml(&vote_sig_url),
            escape_xml(&vote_noise_url)
        ));
        html.push_str("</div>\n</td></tr>\n");
    }

    // Passwordless Magic-Link Footer
    html.push_str("<tr><td style=\"padding-top:18px;font-size:12px;color:#686256;text-align:center;\">\n");
    html.push_str(&format!(
        "<a href=\"{}/\" style=\"color:#1c1a16;text-decoration:underline;\">Read all {} dispatches on Ahbar</a> · ",
        escape_xml(base),
        issue.stats.qualified_items
    ));
    html.push_str(&format!(
        "<a href=\"{}\" style=\"color:#a8321f;font-weight:600;text-decoration:underline;\">Manage Tags &amp; Preferences (No Login)</a> · ",
        escape_xml(&prefs_url)
    ));
    html.push_str(&format!(
        "<a href=\"{}\" style=\"color:#686256;text-decoration:underline;\">One-Click Unsubscribe</a>\n",
        escape_xml(&unsub_url)
    ));
    html.push_str("</td></tr>\n</table>\n</body>\n</html>\n");
    html
}

/// Dispatches an email via Resend Free Tier API or Brevo Free Tier API.
pub async fn dispatch_email_via_provider(
    client: &Client,
    to_email: &str,
    subject: &str,
    html_body: &str,
) -> Result<String, String> {
    if let Ok(resend_key) = env::var("RESEND_API_KEY") {
        if !resend_key.is_empty() {
            let from_addr = env::var("AHBAR_FROM_EMAIL")
                .unwrap_or_else(|_| "Ahbar Digest <onboarding@resend.dev>".to_string());
            let payload = serde_json::json!({
                "from": from_addr,
                "to": [to_email],
                "subject": subject,
                "html": html_body
            });
            let resp = client
                .post("https://api.resend.com/emails")
                .bearer_auth(resend_key)
                .json(&payload)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            if resp.status().is_success() {
                return Ok("resend".to_string());
            }
        }
    }

    if let Ok(brevo_key) = env::var("BREVO_API_KEY") {
        if !brevo_key.is_empty() {
            let from_email = env::var("AHBAR_FROM_ADDRESS")
                .unwrap_or_else(|_| "digest@ahbar.dev".to_string());
            let payload = serde_json::json!({
                "sender": { "name": "Ahbar Digest", "email": from_email },
                "to": [{ "email": to_email }],
                "subject": subject,
                "htmlContent": html_body
            });
            let resp = client
                .post("https://api.brevo.com/v3/smtp/email")
                .header("api-key", brevo_key)
                .json(&payload)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            if resp.status().is_success() {
                return Ok("brevo".to_string());
            }
        }
    }

    Err("No email API key (RESEND_API_KEY or BREVO_API_KEY) configured".to_string())
}

fn urlencoding_simple(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{:02X}", b));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use ahbar_shared::IssueStats;

    #[test]
    fn test_email_renders_directly_without_fluff() {
        let issue = DailyIssue {
            issue_number: 1,
            date: "2026-10-19".to_string(),
            edition_type: EditionType::Daily,
            generated_at: "2026-10-19T05:00:00Z".to_string(),
            stats: IssueStats::default(),
            items: Vec::new(),
        };
        let html = render_digest_email_html(
            &issue,
            &[],
            "reader@example.com",
            "https://latifkedi.github.io/ahbar",
            "secret",
        );
        assert!(html.contains("Ahbar"));
        assert!(html.contains("Manage Tags &amp; Preferences (No Login)"));
        assert!(html.contains("One-Click Unsubscribe"));
    }
}
