use crate::emit::escape_xml;
use ahbar_shared::{
    latex_to_email_unicode, sanitize_http_url, sign_subscriber_token, Area, ContentFilter,
    DailyIssue, DigestFrequency, DigestMode, EditionType, NewsItem, SubscriptionPreferences,
};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use reqwest::Client;
use std::env;

/// Selects the exact list of items for a subscriber according to their
/// Mode (Standard / Personalized / Hybrid), Density (ThreePerArea = 3 per category = 30 items,
/// Compact = 1 per category = 10 items, Comprehensive = 5 per category = 50 items),
/// Frequency, and ContentFilter (All / PapersOnly / CodeOnly).
pub fn select_items_for_subscriber<'a>(
    issue: &'a DailyIssue,
    prefs: &SubscriptionPreferences,
) -> Vec<&'a NewsItem> {
    match (issue.edition_type, prefs.frequency) {
        (EditionType::Weekend, DigestFrequency::WeekdaysOnly) => return Vec::new(),
        (EditionType::Daily, DigestFrequency::WeekendOnly) => return Vec::new(),
        _ => {}
    }

    let per_area = prefs.density.items_per_area();

    let matches_content_filter = |item: &NewsItem| -> bool {
        match prefs.content_filter {
            ContentFilter::All => true,
            ContentFilter::PapersOnly => item.is_paper,
            ContentFilter::CodeOnly => item.links.code_url.is_some(),
        }
    };

    let mut selected: Vec<&'a NewsItem> = Vec::new();

    match prefs.mode {
        DigestMode::Standard | DigestMode::Hybrid => {
            // Select `per_area` (default 3) highest-scoring items from every one of the 10 areas (30 items total)
            for area in Area::ALL {
                let mut count = 0;
                for item in issue.items.iter().filter(|i| i.area == area) {
                    if matches_content_filter(item) {
                        selected.push(item);
                        count += 1;
                        if count >= per_area {
                            break;
                        }
                    }
                }
            }
        }
        DigestMode::Personalized => {
            let active_areas = if prefs.areas.is_empty() {
                Area::ALL.to_vec()
            } else {
                prefs.areas.clone()
            };
            for area in active_areas {
                let mut count = 0;
                for item in issue.items.iter().filter(|i| i.area == area) {
                    let tag_ok = prefs.tags.is_empty()
                        || item
                            .tags
                            .iter()
                            .any(|t| prefs.tags.iter().any(|pt| pt == t));
                    if matches_content_filter(item) && tag_ok {
                        selected.push(item);
                        count += 1;
                        if count >= per_area {
                            break;
                        }
                    }
                }
            }
        }
    }

    selected
}

/// Renders the editorial HTML email grouped by the 10 scientific/technical/strategic categories
/// (3 dispatches per category = 30 dispatches by default), with a top Roman-numeral quick-jump bar (`I` – `X`).
#[allow(dead_code)]
pub fn render_digest_email_html(
    issue: &DailyIssue,
    items: &[&NewsItem],
    subscriber_email: &str,
    site_base_url: &str,
    hmac_secret: &str,
) -> String {
    render_digest_email_html_with_lang(
        issue,
        items,
        subscriber_email,
        site_base_url,
        hmac_secret,
        "en",
    )
}

pub fn render_digest_email_html_with_lang(
    issue: &DailyIssue,
    items: &[&NewsItem],
    subscriber_email: &str,
    site_base_url: &str,
    hmac_secret: &str,
    lang: &str,
) -> String {
    let is_tr = lang.eq_ignore_ascii_case("tr");
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

    let edition_badge = match (issue.edition_type, is_tr) {
        (EditionType::Daily, false) => format!(
            "ISSUE #{} · {} · {} DISPATCHES (10×3)",
            issue.issue_number,
            issue.date,
            items.len()
        ),
        (EditionType::Daily, true) => format!(
            "SAYI #{} · {} · {} BÜLTEN KAYDI (10×3)",
            issue.issue_number,
            issue.date,
            items.len()
        ),
        (EditionType::Weekend, false) => format!(
            "WEEKEND EDITION #{} · {} · {} DISPATCHES",
            issue.issue_number,
            issue.date,
            items.len()
        ),
        (EditionType::Weekend, true) => format!(
            "HAFTA SONU SAYISI #{} · {} · {} KAYIT",
            issue.issue_number,
            issue.date,
            items.len()
        ),
    };

    let mut html = String::with_capacity(49152);
    html.push_str(&format!(
        "<!DOCTYPE html>\n<html lang=\"{}\">\n<head>\n<meta charset=\"utf-8\">\n",
        if is_tr { "tr" } else { "en" }
    ));
    html.push_str("<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\n");
    html.push_str(&format!(
        "<title>Ahbar · {}</title>\n</head>\n",
        escape_xml(&edition_badge)
    ));
    html.push_str("<body style=\"margin:0;padding:28px 16px;background-color:#f6f1e7;color:#1c1a16;font-family:system-ui,-apple-system,'Segoe UI',Roboto,'Helvetica Neue',Arial,sans-serif;line-height:1.6;\">\n");
    html.push_str("<table role=\"presentation\" width=\"100%\" cellpadding=\"0\" cellspacing=\"0\" style=\"max-width:740px;margin:0 auto;background-color:#f6f1e7;\">\n");

    // Top Masthead Bar (useful-sites aesthetic: Enso mark + Ahbar + Issue Telemetry)
    html.push_str("<tr><td style=\"padding-bottom:14px;border-bottom:1px solid #ddd5c4;\">\n");
    html.push_str("<table role=\"presentation\" width=\"100%\" cellpadding=\"0\" cellspacing=\"0\"><tr>\n");
    html.push_str(&format!(
        "<td><a href=\"{}/\" style=\"font-family:'Source Serif 4',Georgia,'Times New Roman',serif;font-size:20px;font-weight:600;color:#1c1a16;text-decoration:none;letter-spacing:-0.01em;\"><span style=\"color:#a8321f;font-weight:700;\">◯</span> Ahbar</a></td>\n",
        escape_xml(base)
    ));
    html.push_str(&format!(
        "<td align=\"right\" style=\"font-size:12px;font-weight:600;letter-spacing:0.12em;color:#a8321f;text-transform:uppercase;\">{}</td>\n",
        escape_xml(&edition_badge)
    ));
    html.push_str("</tr></table>\n</td></tr>\n");

    // Roman-Numeral Quick-Jump Index Bar (I – X)
    html.push_str("<tr><td style=\"padding:10px 0;border-bottom:1px solid #ddd5c4;font-size:12.5px;line-height:1.8;color:#686256;\">\n");
    let mut jump_links = Vec::new();
    for area in Area::ALL {
        let has_any = items.iter().any(|i| i.area == area);
        if has_any {
            jump_links.push(format!(
                "<a href=\"#area-{}\" style=\"color:#1c1a16;text-decoration:none;white-space:nowrap;\"><span style=\"font-family:'Source Serif 4',Georgia,serif;font-weight:600;color:#a8321f;\">{}</span> {}</a>",
                escape_xml(area.slug()),
                escape_xml(area.numeral()),
                escape_xml(area.short_label(is_tr))
            ));
        }
    }
    html.push_str(&jump_links.join(" <span style=\"color:#ddd5c4;margin:0 5px;\">·</span> "));
    html.push_str("\n</td></tr>\n");

    // Showcase: Trending Repos, Open Weights & Applied Tech (Top 3 Dispatches)
    let trending_picks: Vec<&&NewsItem> = items
        .iter()
        .filter(|i| {
            i.tags.iter().any(|t| {
                matches!(
                    t.as_str(),
                    "trending-repo"
                        | "ai-models"
                        | "open-weights"
                        | "open-source"
                        | "consumer-tech"
                        | "hardware"
                )
            }) || i.links.code_url.is_some()
        })
        .take(3)
        .collect();

    if !trending_picks.is_empty() {
        let showcase_title = if is_tr {
            "🔥 HAFTANIN TREND AÇIK KAYNAK PROJELERİ & YENİ AI MODELLERİ"
        } else {
            "🔥 TRENDING OPEN-SOURCE REPOSITORIES & AI MODEL RELEASES"
        };
        let showcase_sub = if is_tr {
            "Geliştirici araçları, açık modeller ve halka yansıyan yeni donanımlar"
        } else {
            "Trending developer tools, open weights & applied hardware"
        };

        html.push_str("<tr><td style=\"padding:18px 0 6px 0;\">\n");
        html.push_str("<table role=\"presentation\" width=\"100%\" cellpadding=\"0\" cellspacing=\"0\" style=\"background-color:#eee7d8;border-left:4px solid #a8321f;border-radius:4px;padding:14px 16px;\"><tr><td>\n");
        html.push_str(&format!(
            "<div style=\"font-size:12px;font-weight:700;letter-spacing:0.06em;color:#a8321f;text-transform:uppercase;\">{}</div>\n",
            escape_xml(showcase_title)
        ));
        html.push_str(&format!(
            "<div style=\"font-size:12px;color:#686256;margin:2px 0 12px 0;\">{}</div>\n",
            escape_xml(showcase_sub)
        ));

        for tp in &trending_picks {
            let tp_head = latex_to_email_unicode(tp.headline_tr.as_deref().unwrap_or(&tp.headline));
            let tp_sum = latex_to_email_unicode(tp.simple_summary(lang));
            let tp_url = if is_tr {
                format!("{}/?lang=tr&item={}", base, tp.id)
            } else {
                format!("{}/?item={}", base, tp.id)
            };

            html.push_str("<div style=\"margin-bottom:10px;padding-bottom:10px;border-bottom:1px dashed #ddd5c4;\">\n");
            html.push_str(&format!(
                "<a href=\"{}\" style=\"font-family:'Source Serif 4',Georgia,serif;font-size:15px;font-weight:600;color:#1c1a16;text-decoration:none;\">{}</a>\n",
                escape_xml(&tp_url),
                escape_xml(&tp_head)
            ));
            html.push_str(&format!(
                "<p style=\"margin:3px 0 5px 0;font-size:13.5px;line-height:1.55;color:#5b564c;\">{}</p>\n",
                escape_xml(&tp_sum)
            ));
            if let Some(ref code) = tp.links.code_url {
                html.push_str(&format!(
                    "<a href=\"{}\" style=\"font-size:11.5px;font-weight:600;color:#a8321f;text-decoration:underline;margin-right:12px;\">GitHub Kaynak Kodu ↗</a>",
                    escape_xml(code)
                ));
            }
            html.push_str(&format!(
                "<a href=\"{}\" style=\"font-size:11.5px;color:#686256;text-decoration:underline;\">Ahbar Analizi →</a>\n",
                escape_xml(&tp_url)
            ));
            html.push_str("</div>\n");
        }

        html.push_str("</td></tr></table>\n</td></tr>\n");
    }

    // Group items by the 10 Areas (3 items per area = 30 items total)
    let mut global_seq = 1usize;
    for area in Area::ALL {
        let area_items: Vec<&&NewsItem> = items.iter().filter(|i| i.area == area).collect();
        if area_items.is_empty() {
            continue;
        }

        let area_title = if is_tr {
            area.title_tr()
        } else {
            area.title()
        };
        let area_sub = if is_tr {
            area.subtitle_tr()
        } else {
            area.subtitle()
        };

        // Category Header Row (Roman Numeral + Area Title + Count)
        html.push_str(&format!(
            "<tr><td id=\"area-{}\" style=\"padding:26px 0 8px 0;border-bottom:1px solid #ddd5c4;\">\n",
            escape_xml(area.slug())
        ));
        html.push_str(&format!(
            "<div style=\"font-family:'Source Serif 4',Georgia,'Times New Roman',serif;font-size:20px;font-weight:600;color:#1c1a16;letter-spacing:-0.01em;\"><span style=\"color:#a8321f;margin-right:8px;\">{}</span>{} <span style=\"font-family:system-ui,sans-serif;font-size:12px;font-weight:400;color:#686256;margin-left:6px;\">({})</span></div>\n",
            escape_xml(area.numeral()),
            escape_xml(area_title),
            area_items.len()
        ));
        html.push_str(&format!(
            "<div style=\"font-size:13px;color:#5b564c;margin-top:2px;\">{}</div>\n",
            escape_xml(area_sub)
        ));
        html.push_str("</td></tr>\n");

        // The 3 items in this Area
        for item in area_items {
            let num = global_seq;
            global_seq += 1;

            let detail_url = if is_tr {
                format!("{}/?lang=tr&item={}", base, item.id)
            } else {
                format!("{}/?item={}", base, item.id)
            };
            let safe_item_url =
                sanitize_http_url(&item.url).unwrap_or_else(|| detail_url.clone());
            let wayback_url = format!("https://web.archive.org/web/*/{}", safe_item_url);
            let vote_sig_url = format!("{}/?vote=signal&item={}", base, item.id);
            let vote_noise_url = format!("{}/?vote=noise&item={}", base, item.id);

            let headline_raw = if is_tr {
                item.headline_tr.as_deref().unwrap_or(&item.headline)
            } else {
                &item.headline
            };
            let summary_raw = item.simple_summary(lang);

            let headline_clean = latex_to_email_unicode(headline_raw);
            let summary_clean = latex_to_email_unicode(summary_raw);

            let mut badges_html = String::new();
            if item.links.code_url.is_some() {
                badges_html.push_str(if is_tr {
                    " <span style=\"background-color:#eee7d8;color:#a8321f;border:1px solid #ddd5c4;border-radius:3px;padding:1px 5px;font-size:11px;font-weight:600;\">Açık Kod</span>"
                } else {
                    " <span style=\"background-color:#eee7d8;color:#a8321f;border:1px solid #ddd5c4;border-radius:3px;padding:1px 5px;font-size:11px;font-weight:600;\">Code</span>"
                });
            }
            if item.tags.iter().any(|t| t == "consumer-tech" || t == "hardware") {
                badges_html.push_str(if is_tr {
                    " <span style=\"background-color:#eee7d8;color:#a8321f;border:1px solid #ddd5c4;border-radius:3px;padding:1px 5px;font-size:11px;font-weight:600;\">Tüketici Teknoloji</span>"
                } else {
                    " <span style=\"background-color:#eee7d8;color:#a8321f;border:1px solid #ddd5c4;border-radius:3px;padding:1px 5px;font-size:11px;font-weight:600;\">Hardware</span>"
                });
            }
            if item.tags.iter().any(|t| t == "open-weights" || t == "ai-models") {
                badges_html.push_str(if is_tr {
                    " <span style=\"background-color:#eee7d8;color:#a8321f;border:1px solid #ddd5c4;border-radius:3px;padding:1px 5px;font-size:11px;font-weight:600;\">Açık Model</span>"
                } else {
                    " <span style=\"background-color:#eee7d8;color:#a8321f;border:1px solid #ddd5c4;border-radius:3px;padding:1px 5px;font-size:11px;font-weight:600;\">Open Model</span>"
                });
            }

            html.push_str("<tr><td style=\"padding:16px 0;border-bottom:1px solid #e9e2d3;\">\n");
            html.push_str("<table role=\"presentation\" width=\"100%\" cellpadding=\"0\" cellspacing=\"0\"><tr>\n");

            // Left column: tabular serif sequence number in red (matches .rec .no in useful-sites)
            html.push_str(&format!(
                "<td width=\"32\" valign=\"top\" style=\"font-family:'Source Serif 4',Georgia,serif;font-size:13.5px;font-weight:600;color:#a8321f;padding-right:10px;padding-top:3px;text-align:right;\">{:02}</td>\n",
                num
            ));

            // Right column: headline, source host, 1-paragraph summary, tags & direct links
            html.push_str("<td valign=\"top\">\n");
            html.push_str(&format!(
                "<div style=\"margin:0 0 5px 0;\"><a href=\"{}\" style=\"font-family:'Source Serif 4',Georgia,'Times New Roman',serif;font-size:18px;font-weight:600;line-height:1.32;color:#1c1a16;text-decoration:none;\">{}</a>{} <span style=\"font-size:12.5px;color:#686256;margin-left:6px;\">· {}</span></div>\n",
                escape_xml(&detail_url),
                escape_xml(&headline_clean),
                badges_html,
                escape_xml(&item.source_name)
            ));

            html.push_str(&format!(
                "<p style=\"margin:0 0 8px 0;font-size:15px;line-height:1.62;color:#5b564c;\"><strong style=\"color:#a8321f;font-size:12px;letter-spacing:0.04em;\">💡 {}</strong> {}</p>\n",
                if is_tr { "ÖZET:" } else { "SUMMARY:" },
                escape_xml(&summary_clean)
            ));

            // Metadata bar (links + inline tags)
            html.push_str("<div style=\"font-size:12.5px;color:#686256;\">\n");
            let tech_lbl = if is_tr { "🔬 Teknik Analiz Oku →" } else { "🔬 Technical Detail →" };
            let source_lbl = if is_tr { "Kaynak ↗" } else { "Source ↗" };
            let arch_lbl = if is_tr { "arşiv" } else { "archive" };
            html.push_str(&format!(
                "<a href=\"{}\" style=\"color:#a8321f;font-weight:600;text-decoration:none;margin-right:12px;\">{}</a>",
                escape_xml(&detail_url),
                tech_lbl
            ));
            html.push_str(&format!(
                "<a href=\"{}\" style=\"color:#5b564c;text-decoration:none;border-bottom:1px dotted #ddd5c4;margin-right:12px;\">{}</a>",
                escape_xml(&safe_item_url),
                source_lbl
            ));
            if let Some(ref pdf) = item.links.pdf_url {
                if let Some(safe_pdf) = sanitize_http_url(pdf) {
                    html.push_str(&format!(
                        "<a href=\"{}\" style=\"color:#5b564c;text-decoration:none;border-bottom:1px dotted #ddd5c4;margin-right:12px;\">PDF</a>",
                        escape_xml(&safe_pdf)
                    ));
                }
            }
            if let Some(ref ar5iv) = item.links.html_ar5iv_url {
                if let Some(safe_ar5iv) = sanitize_http_url(ar5iv) {
                    html.push_str(&format!(
                        "<a href=\"{}\" style=\"color:#5b564c;text-decoration:none;border-bottom:1px dotted #ddd5c4;margin-right:12px;\">HTML (ar5iv)</a>",
                        escape_xml(&safe_ar5iv)
                    ));
                }
            }
            if let Some(ref code) = item.links.code_url {
                if let Some(safe_code) = sanitize_http_url(code) {
                    html.push_str(&format!(
                        "<a href=\"{}\" style=\"color:#5b564c;text-decoration:none;border-bottom:1px dotted #ddd5c4;margin-right:12px;\">Code</a>",
                        escape_xml(&safe_code)
                    ));
                }
            }
            html.push_str(&format!(
                "<a href=\"{}\" style=\"color:#686256;text-decoration:none;border-bottom:1px dotted #ddd5c4;margin-right:12px;\">{}</a>",
                escape_xml(&wayback_url),
                arch_lbl
            ));
            for (t_idx, tag) in item.tags.iter().enumerate() {
                if t_idx > 0 {
                    html.push_str("<span style=\"color:#ddd5c4;margin:0 4px;\">·</span>");
                }
                html.push_str(&format!(
                    "<span style=\"color:#5b564c;\">#{}</span>",
                    escape_xml(tag)
                ));
            }
            html.push_str(&format!(
                "<span style=\"float:right;\"><a href=\"{}\" style=\"color:#686256;text-decoration:none;margin-right:8px;\">▲</a><a href=\"{}\" style=\"color:#686256;text-decoration:none;\">▼</a></span>",
                escape_xml(&vote_sig_url),
                escape_xml(&vote_noise_url)
            ));
            html.push_str("</div>\n</td></tr></table>\n</td></tr>\n");
        }
    }

    // Passwordless Magic-Link Footer
    html.push_str("<tr><td style=\"padding-top:24px;font-size:13px;color:#686256;\">\n");
    if is_tr {
        html.push_str(&format!(
            "<a href=\"{}/?lang=tr&amp;all=1\" style=\"color:#1c1a16;text-decoration:underline;\">Ahbar üzerindeki tüm {} kaydı gör →</a> · ",
            escape_xml(base),
            issue.stats.qualified_items
        ));
        html.push_str(&format!(
            "<a href=\"{}/k/\" style=\"color:#5b564c;text-decoration:underline;\">Düz metin dizini (k/)</a> · ",
            escape_xml(base)
        ));
        html.push_str(&format!(
            "<a href=\"{}\" style=\"color:#a8321f;font-weight:600;text-decoration:underline;\">Etiket &amp; Tercih Yönetimi (Şifresiz)</a> · ",
            escape_xml(&prefs_url)
        ));
        html.push_str(&format!(
            "<a href=\"{}\" style=\"color:#686256;text-decoration:underline;\">Tek Tıkla Abonelikten Çık</a>\n",
            escape_xml(&unsub_url)
        ));
    } else {
        html.push_str(&format!(
            "<a href=\"{}/?all=1\" style=\"color:#1c1a16;text-decoration:underline;\">Read all {} dispatches in the full index →</a> · ",
            escape_xml(base),
            issue.stats.qualified_items
        ));
        html.push_str(&format!(
            "<a href=\"{}/k/en/\" style=\"color:#5b564c;text-decoration:underline;\">Plain-text index (k/)</a> · ",
            escape_xml(base)
        ));
        html.push_str(&format!(
            "<a href=\"{}\" style=\"color:#a8321f;font-weight:600;text-decoration:underline;\">Manage Tags &amp; Preferences (No Login)</a> · ",
            escape_xml(&prefs_url)
        ));
        html.push_str(&format!(
            "<a href=\"{}\" style=\"color:#686256;text-decoration:underline;\">One-Click Unsubscribe</a>\n",
            escape_xml(&unsub_url)
        ));
    }
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
            } else {
                let status = resp.status();
                let err_text = resp.text().await.unwrap_or_default();
                return Err(format!("Resend API error (HTTP {}): {}", status, err_text));
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
            } else {
                let status = resp.status();
                let err_text = resp.text().await.unwrap_or_default();
                return Err(format!("Brevo API error (HTTP {}): {}", status, err_text));
            }
        }
    }

    // Direct SMTP fallback (e.g. Gmail SMTP with App Password, custom mailserver, or self-hosted)
    if let (Ok(smtp_user), Ok(smtp_pass)) = (env::var("SMTP_USER"), env::var("SMTP_PASS")) {
        if !smtp_user.is_empty() && !smtp_pass.is_empty() {
            let smtp_host = env::var("SMTP_HOST").unwrap_or_else(|_| {
                if smtp_user.ends_with("@gmail.com") {
                    "smtp.gmail.com".to_string()
                } else {
                    "localhost".to_string()
                }
            });
            let smtp_port: u16 = env::var("SMTP_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(587);

            let from_header = env::var("AHBAR_FROM_EMAIL").unwrap_or_else(|_| {
                format!("Ahbar Bülten <{}>", smtp_user)
            });

            let email_res = Message::builder()
                .from(match from_header.parse() {
                    Ok(f) => f,
                    Err(_) => match smtp_user.parse() {
                        Ok(u) => u,
                        Err(e) => return Err(format!("Invalid SMTP sender address: {}", e)),
                    },
                })
                .to(to_email.parse().map_err(|e| format!("Invalid recipient email '{}': {}", to_email, e))?)
                .subject(subject)
                .header(ContentType::TEXT_HTML)
                .body(html_body.to_string());

            let email_msg = match email_res {
                Ok(m) => m,
                Err(e) => return Err(format!("Failed to build SMTP email message: {}", e)),
            };

            let creds = Credentials::new(smtp_user.clone(), smtp_pass.replace(' ', ""));
            let transport_res: Result<AsyncSmtpTransport<Tokio1Executor>, _> = if smtp_port == 465 {
                AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_host)
                    .map(|b| b.credentials(creds).port(smtp_port).build())
            } else {
                AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)
                    .map(|b| b.credentials(creds).port(smtp_port).build())
            };

            let transport = match transport_res {
                Ok(t) => t,
                Err(e) => return Err(format!("Failed to configure SMTP transport for {}: {}", smtp_host, e)),
            };

            match transport.send(email_msg).await {
                Ok(_) => return Ok("smtp".to_string()),
                Err(e) => return Err(format!("SMTP delivery error via {}: {}", smtp_host, e)),
            }
        }
    }

    Err("No email provider configured (set RESEND_API_KEY, BREVO_API_KEY, or SMTP_USER + SMTP_PASS)".to_string())
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

/// Builds the HTML formatted text for Telegram Bot broadcast.
pub fn build_telegram_digest_text(issue: &DailyIssue, site_base_url: &str) -> String {
    let mut text = format!(
        "⚡ <b>Ahbar — Günlük Bilim &amp; Teknoloji Seçkisi</b> (Sayı #{})\n<i>{} · {} Nitelikli Kayıt</i>\n\n",
        issue.issue_number, issue.date, issue.stats.qualified_items
    );

    let mut picked = Vec::new();
    for area in Area::ALL {
        if let Some(item) = issue
            .items
            .iter()
            .find(|i| i.area == area && i.is_top_pick)
            .or_else(|| issue.items.iter().find(|i| i.area == area))
        {
            picked.push(item);
        }
    }

    for item in picked.iter().take(10) {
        let title = item.headline_tr.as_deref().unwrap_or(&item.headline);
        let summary = item.summary_tr.as_deref().unwrap_or(&item.summary);
        let safe_url = sanitize_http_url(&item.url)
            .unwrap_or_else(|| "https://latifkedi.github.io/ahbar".to_string());
        let item_link = format!("{}/?item={}", site_base_url.trim_end_matches('/'), item.id);

        text.push_str(&format!(
            "<b>{} {}</b> · <a href=\"{}\">{}</a>\n{}\n<a href=\"{}\">Detay &amp; Analiz →</a> · <a href=\"{}\">Kaynak ↗</a>\n\n",
            item.area.numeral(),
            escape_xml(item.area.title_tr()),
            escape_xml(&item_link),
            escape_xml(title),
            escape_xml(summary),
            escape_xml(&item_link),
            escape_xml(&safe_url)
        ));
    }

    text.push_str(&format!(
        "🌐 <a href=\"{}\">Tüm bülteni web üzerinde oku ({}) →</a>",
        escape_xml(site_base_url),
        issue.items.len()
    ));

    if text.chars().count() > 3900 {
        let mut truncated: String = text.chars().take(3800).collect();
        truncated.push_str(&format!(
            "\n\n... <a href=\"{}\">Devamını web üzerinde oku →</a>",
            escape_xml(site_base_url)
        ));
        text = truncated;
    }

    text
}

/// Dispatches a high-priority summary of the daily issue to Telegram via Bot API.
pub async fn dispatch_telegram_digest(
    client: &Client,
    issue: &DailyIssue,
    bot_token: &str,
    chat_id: &str,
    site_base_url: &str,
) -> Result<String, String> {
    let text = build_telegram_digest_text(issue, site_base_url);
    let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);
    let payload = serde_json::json!({
        "chat_id": chat_id,
        "text": text,
        "parse_mode": "HTML",
        "disable_web_page_preview": true
    });

    let resp = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let err_body = resp.text().await.unwrap_or_default();
        return Err(format!("Telegram API error: {}", err_body));
    }

    Ok("telegram_ok".to_string())
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

    #[test]
    fn test_telegram_digest_formatting() {
        let issue = DailyIssue {
            issue_number: 42,
            date: "2026-10-19".to_string(),
            edition_type: EditionType::Daily,
            generated_at: "2026-10-19T05:00:00Z".to_string(),
            stats: IssueStats {
                qualified_items: 25,
                ..Default::default()
            },
            items: Vec::new(),
        };
        let text = build_telegram_digest_text(&issue, "https://latifkedi.github.io/ahbar");
        assert!(text.contains("Ahbar — Günlük Bilim &amp; Teknoloji Seçkisi"));
        assert!(text.contains("Sayı #42"));
    }
}
