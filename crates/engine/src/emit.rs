use ahbar_shared::{
    latex_to_html_math, ArchiveEntry, Area, DailyIssue, NewsItem, SourceHealthRecord,
    TechnicalDetail, CANON_TAGS,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Writes the chunked static JSON payloads (`data/latest.json`, `data/latest.en.json`, `data/latest.tr.json`,
/// `data/issues/<date>.json`, `data/issues/<date>.en.json`, `data/issues/<date>.tr.json`,
/// `data/archive.json`, `data/health.json`, `data/tags.json`), generates static Atom 1.0
/// feeds (`feed/all.xml`, `feed/area-<slug>.xml`, `feed/tag-<slug>.xml`), generates the plain-text
/// static editions (`k/index.html` and `k/en/index.html`), and updates `index.html` with inlined CSS
/// and pre-rendered `<div class="home" data-pre="1">` for 0ms first paint.
pub fn emit_all(
    root_dir: &Path,
    issue: &DailyIssue,
    health: &[SourceHealthRecord],
    site_base_url: &str,
) -> Result<(), String> {
    let data_dir = root_dir.join("data");
    let issues_dir = data_dir.join("issues");
    let feed_dir = root_dir.join("feed");
    let k_tr_dir = root_dir.join("k");
    let k_en_dir = k_tr_dir.join("en");

    fs::create_dir_all(&issues_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&feed_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&k_en_dir).map_err(|e| e.to_string())?;

    // 1. Write full bilingual data/latest.json + data/issues/<date>.json
    let latest_json = serde_json::to_string_pretty(issue).map_err(|e| e.to_string())?;
    fs::write(data_dir.join("latest.json"), &latest_json).map_err(|e| e.to_string())?;
    fs::write(issues_dir.join(format!("{}.json", issue.date)), &latest_json)
        .map_err(|e| e.to_string())?;

    // 2. Write language-split payloads (latest.en.json / latest.tr.json + issues/<date>.en.json / .tr.json)
    let issue_en = build_localized_issue(issue, false);
    let issue_tr = build_localized_issue(issue, true);
    let json_en = serde_json::to_string(&issue_en).map_err(|e| e.to_string())?;
    let json_tr = serde_json::to_string(&issue_tr).map_err(|e| e.to_string())?;

    fs::write(data_dir.join("latest.en.json"), &json_en).map_err(|e| e.to_string())?;
    fs::write(data_dir.join("latest.tr.json"), &json_tr).map_err(|e| e.to_string())?;
    fs::write(
        issues_dir.join(format!("{}.en.json", issue.date)),
        &json_en,
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        issues_dir.join(format!("{}.tr.json", issue.date)),
        &json_tr,
    )
    .map_err(|e| e.to_string())?;

    // 3. Update data/archive.json
    update_archive_index(&data_dir, issue)?;

    // 4. Write data/health.json
    let health_json = serde_json::to_string_pretty(health).map_err(|e| e.to_string())?;
    fs::write(data_dir.join("health.json"), health_json).map_err(|e| e.to_string())?;

    // 5. Write data/tags.json (canonical tags metadata for frontend filter chips)
    emit_tags_metadata(&data_dir)?;

    // 6. Emit Atom 1.0 XML feeds (root feed.xml + feed/all.xml + per-area + per-tag)
    let all_atom = build_atom_feed(
        "Ahbar — All Technical, Scientific & Strategic Dispatches",
        "Daily curated dispatches across computer science, AI, mathematics, physics, systems, biotech, cryptography, economics, geopolitics, and philosophy of science.",
        site_base_url,
        "feed/all.xml",
        &issue.generated_at,
        &issue.items,
    );
    fs::write(feed_dir.join("all.xml"), &all_atom).map_err(|e| e.to_string())?;
    fs::write(root_dir.join("feed.xml"), &all_atom).map_err(|e| e.to_string())?;

    // Per-area feeds across all 10 areas
    for area in Area::ALL {
        let area_items: Vec<NewsItem> = issue
            .items
            .iter()
            .filter(|i| i.area == area)
            .cloned()
            .collect();
        let rel_path = format!("feed/area-{}.xml", area.slug());
        let xml = build_atom_feed(
            &format!("Ahbar · {}. {}", area.numeral(), area.title()),
            area.subtitle(),
            site_base_url,
            &rel_path,
            &issue.generated_at,
            &area_items,
        );
        fs::write(root_dir.join(&rel_path), xml).map_err(|e| e.to_string())?;
    }

    // Per-tag feeds for canonical tags
    let mut by_tag: HashMap<String, Vec<NewsItem>> = HashMap::new();
    for item in &issue.items {
        for tag in &item.tags {
            by_tag.entry(tag.clone()).or_default().push(item.clone());
        }
    }

    for canon in CANON_TAGS {
        let items = by_tag.get(canon.slug).cloned().unwrap_or_default();
        if !items.is_empty() {
            let rel_path = format!("feed/tag-{}.xml", canon.slug);
            let xml = build_atom_feed(
                &format!("Ahbar · #{}", canon.label),
                &format!("Daily technical dispatches tagged with #{}.", canon.slug),
                site_base_url,
                &rel_path,
                &issue.generated_at,
                &items,
            );
            fs::write(root_dir.join(&rel_path), xml).map_err(|e| e.to_string())?;
        }
    }

    // 7. Emit plain-text static editions: k/index.html (TR) and k/en/index.html (EN)
    let static_tr = build_static_directory_html(issue, true);
    let static_en = build_static_directory_html(issue, false);
    fs::write(k_tr_dir.join("index.html"), static_tr).map_err(|e| e.to_string())?;
    fs::write(k_en_dir.join("index.html"), static_en).map_err(|e| e.to_string())?;

    // 8. Pre-render `<div class="home" data-pre="1">` and inline `style.css` into `index.html`
    inject_prerender_into_index_html(root_dir, issue)?;

    Ok(())
}

/// Produces a lightweight single-language `DailyIssue` payload where `headline`, `summary`,
/// and `detail` are already localized for that language and redundant `_tr` duplicate fields are stripped.
fn build_localized_issue(issue: &DailyIssue, is_tr: bool) -> DailyIssue {
    let items = issue
        .items
        .iter()
        .map(|item| {
            let headline = if is_tr {
                item.headline_tr
                    .clone()
                    .unwrap_or_else(|| item.headline.clone())
            } else {
                item.headline.clone()
            };
            let summary = if is_tr {
                item.summary_tr
                    .clone()
                    .unwrap_or_else(|| item.summary.clone())
            } else {
                item.summary.clone()
            };
            let problem_context = if is_tr {
                item.detail
                    .problem_context_tr
                    .clone()
                    .unwrap_or_else(|| item.detail.problem_context.clone())
            } else {
                item.detail.problem_context.clone()
            };
            let method_architecture = if is_tr {
                item.detail
                    .method_architecture_tr
                    .clone()
                    .unwrap_or_else(|| item.detail.method_architecture.clone())
            } else {
                item.detail.method_architecture.clone()
            };
            let findings_tradeoffs = if is_tr {
                item.detail
                    .findings_tradeoffs_tr
                    .clone()
                    .unwrap_or_else(|| item.detail.findings_tradeoffs.clone())
            } else {
                item.detail.findings_tradeoffs.clone()
            };

            NewsItem {
                id: item.id.clone(),
                issue_date: item.issue_date.clone(),
                issue_number: item.issue_number,
                area: item.area,
                is_top_pick: item.is_top_pick,
                is_paper: item.is_paper,
                headline,
                summary,
                headline_tr: None,
                summary_tr: None,
                detail: TechnicalDetail {
                    problem_context,
                    method_architecture,
                    findings_tradeoffs,
                    problem_context_tr: None,
                    method_architecture_tr: None,
                    findings_tradeoffs_tr: None,
                    abstract_excerpt: item.detail.abstract_excerpt.clone(),
                    bibtex: item.detail.bibtex.clone(),
                },
                original_title: item.original_title.clone(),
                authors: item.authors.clone(),
                source_name: item.source_name.clone(),
                url: item.url.clone(),
                links: item.links.clone(),
                tags: item.tags.clone(),
                metrics: item.metrics.clone(),
                published_at: item.published_at.clone(),
            }
        })
        .collect();

    DailyIssue {
        issue_number: issue.issue_number,
        date: issue.date.clone(),
        edition_type: issue.edition_type,
        generated_at: issue.generated_at.clone(),
        stats: issue.stats.clone(),
        items,
    }
}

/// Builds the plain-text static edition (`k/index.html` for TR, `k/en/index.html` for EN)
/// inspired by `useful-sites/k/index.html` (`body.static`).
fn build_static_directory_html(issue: &DailyIssue, is_tr: bool) -> String {
    let css_href = if is_tr { "../style.css" } else { "../../style.css" };
    let home_href = if is_tr { "../?lang=tr" } else { "../../?lang=en" };
    let other_lang_href = if is_tr { "en/" } else { "../" };
    let other_lang_lbl = if is_tr { "EN" } else { "TR" };
    let feed_href = if is_tr { "../feed.xml" } else { "../../feed.xml" };

    let title = if is_tr {
        format!(
            "Ahbar — Sayı #{} ({}) · Düz Metin Dizini",
            issue.issue_number, issue.date
        )
    } else {
        format!(
            "Ahbar — Issue #{} ({}) · Plain-Text Directory",
            issue.issue_number, issue.date
        )
    };

    let mut html = String::with_capacity(65536);
    html.push_str(&format!(
        "<!doctype html>\n<html lang=\"{}\" data-theme=\"light\">\n<head>\n<meta charset=\"utf-8\">\n",
        if is_tr { "tr" } else { "en" }
    ));
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    html.push_str("<meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'\">\n");
    html.push_str(&format!("<title>{}</title>\n", escape_xml(&title)));
    html.push_str(&format!("<link rel=\"stylesheet\" href=\"{}\">\n", css_href));
    html.push_str("</head>\n<body class=\"calm static\">\n<div class=\"wrap\">\n");

    html.push_str("<header class=\"top\">\n");
    html.push_str(&format!(
        "  <a class=\"logo\" href=\"{}\"><svg class=\"enso\" viewBox=\"0 0 60 60\" aria-hidden=\"true\"><path d=\"M43 12A22 22 0 1 0 51 30\"/></svg><span>Ahbar</span></a>\n",
        home_href
    ));
    html.push_str("  <nav class=\"acts\">\n");
    html.push_str(&format!(
        "    <a class=\"lnk\" href=\"{}\">{}</a>\n",
        home_href,
        if is_tr {
            "← Etkileşimli Dizin"
        } else {
            "← Interactive Index"
        }
    ));
    html.push_str(&format!(
        "    <a class=\"lnk\" href=\"{}\">Atom / RSS ↗</a>\n",
        feed_href
    ));
    html.push_str(&format!(
        "    <a class=\"lnk\" href=\"{}\">{}</a>\n",
        other_lang_href, other_lang_lbl
    ));
    html.push_str("  </nav>\n</header>\n\n<main id=\"list\">\n<div class=\"fieldpage\">\n");

    html.push_str(&format!(
        "<h1 class=\"ph\">{} <span class=\"n\">{}</span></h1>\n",
        if is_tr {
            format!("Sayı #{} · {}", issue.issue_number, issue.date)
        } else {
            format!("Issue #{} · {}", issue.issue_number, issue.date)
        },
        issue.items.len()
    ));

    // Jump links for the 10 areas
    html.push_str("<p class=\"lede\">");
    let mut jlinks = Vec::new();
    for area in Area::ALL {
        let a_title = if is_tr { area.title_tr() } else { area.title() };
        jlinks.push(format!(
            "<a href=\"#sec-{}\"><span class=\"rn\">{}</span> {}</a>",
            area.slug(),
            area.numeral(),
            escape_xml(a_title)
        ));
    }
    html.push_str(&jlinks.join(" · "));
    html.push_str("</p>\n");

    let mut seq = 1usize;
    for area in Area::ALL {
        let area_items: Vec<&NewsItem> = issue.items.iter().filter(|i| i.area == area).collect();
        if area_items.is_empty() {
            continue;
        }
        let a_title = if is_tr { area.title_tr() } else { area.title() };
        html.push_str(&format!(
            "<section id=\"sec-{}\">\n<h2 class=\"sh\"><span class=\"rn\">{}</span> {} <span class=\"n\">{}</span></h2>\n<div class=\"recs\">\n",
            area.slug(),
            area.numeral(),
            escape_xml(a_title),
            area_items.len()
        ));

        for item in area_items {
            let head = if is_tr {
                item.headline_tr.as_deref().unwrap_or(&item.headline)
            } else {
                &item.headline
            };
            let sum = if is_tr {
                item.summary_tr.as_deref().unwrap_or(&item.summary)
            } else {
                &item.summary
            };
            let wayback = format!("https://web.archive.org/web/*/{}", item.url);
            html.push_str(&format!(
                "<article class=\"rec\"><div class=\"no\">{:02}</div><div class=\"rb\">",
                seq
            ));
            seq += 1;
            html.push_str(&format!(
                "<div class=\"nm\"><a class=\"name\" href=\"{}\" target=\"_blank\" rel=\"noopener\">{}</a> <span class=\"host\">{}</span></div>",
                escape_xml(&item.url),
                latex_to_html_math(head),
                escape_xml(&item.source_name)
            ));
            html.push_str(&format!("<p class=\"desc\">{}</p>", latex_to_html_math(sum)));
            html.push_str("<div class=\"mt\"><span class=\"itags\">");
            html.push_str(
                &item
                    .tags
                    .iter()
                    .map(|t| format!("#{}", escape_xml(t)))
                    .collect::<Vec<_>>()
                    .join(" · "),
            );
            html.push_str("</span><span class=\"meta\">");
            if let Some(ref pdf) = item.links.pdf_url {
                html.push_str(&format!(
                    "<a class=\"arch\" href=\"{}\" target=\"_blank\" rel=\"noopener\">PDF</a>",
                    escape_xml(pdf)
                ));
            }
            html.push_str(&format!(
                "<a class=\"arch\" href=\"{}\" target=\"_blank\" rel=\"noopener\">{}</a>",
                escape_xml(&wayback),
                if is_tr { "arşiv" } else { "archive" }
            ));
            html.push_str("</span></div></div></article>\n");
        }
        html.push_str("</div>\n</section>\n");
    }

    html.push_str("</div>\n</main>\n</div>\n</body>\n</html>\n");
    html
}

/// Builds the pre-rendered `<div class="home" data-pre="1">` HTML and embeds `style.css`
/// directly into `index.html` between marker comments so the initial paint is 0ms.
pub fn render_prerendered_home_html(issue: &DailyIssue) -> String {
    let mut out = String::with_capacity(32768);
    out.push_str("<div class=\"home\" data-pre=\"1\">");
    out.push_str(&format!(
        "<p class=\"proof\">{} seçilmiş kayıt · {} · {} birincil kaynaktan {} makale ve besleme tarandı · her gün 08:00 TSİ · izleyici yok</p>",
        issue.items.len(),
        escape_xml(&issue.date),
        issue.stats.sources_checked,
        issue.stats.raw_items_scanned
    ));
    out.push_str("<p class=\"tryq\"><span>Dene:</span>");
    for q in [
        "llm",
        "rust",
        "zkp",
        "quantum",
        "lean 4",
        "monetary-policy",
        "geopolitics",
        "epistemology",
    ] {
        out.push_str(&format!(
            "<a class=\"chip\" href=\"?q={}\" data-q=\"{}\">{}</a>",
            escape_xml(q),
            escape_xml(q),
            escape_xml(q)
        ));
    }
    out.push_str("</p>");

    // 1. Top Picks showcase (6 items across categories)
    out.push_str("<section class=\"hsec hp\"><h2 class=\"k\">Günün Seçkisi (Top Picks)<span>her kategoriden 3 haber · e-postada 30 kayıt</span></h2><ol class=\"hpicks\">");
    let mut picks_shown = 0usize;
    for area in Area::ALL {
        if picks_shown >= 6 {
            break;
        }
        if let Some(item) = issue
            .items
            .iter()
            .find(|i| i.area == area && i.is_top_pick)
            .or_else(|| issue.items.iter().find(|i| i.area == area))
        {
            picks_shown += 1;
            let head = item.headline_tr.as_deref().unwrap_or(&item.headline);
            let sum = item.summary_tr.as_deref().unwrap_or(&item.summary);
            out.push_str(&format!(
                "<li><a class=\"hp-title\" href=\"?item={}\" data-item=\"{}\">{}</a><p>{}</p><div class=\"n\" style=\"margin-top:6px;\"><span class=\"rn\">{}</span> · {} · {}</div></li>",
                escape_xml(&item.id),
                escape_xml(&item.id),
                latex_to_html_math(head),
                latex_to_html_math(sum),
                area.numeral(),
                escape_xml(area.title_tr()),
                escape_xml(&item.source_name)
            ));
        }
    }
    out.push_str("</ol></section>");

    // 2. Fihrist (10 areas in 2 columns)
    out.push_str(&format!(
        "<section class=\"hsec\"><h2 class=\"k\">Fihrist<span>10 alan, {} kanonik etiket</span></h2><ol class=\"fx\">",
        CANON_TAGS.len()
    ));
    for area in Area::ALL {
        let cnt = issue.items.iter().filter(|i| i.area == area).count();
        let area_tags: Vec<String> = CANON_TAGS
            .iter()
            .filter(|t| t.area_slug == area.slug())
            .take(6)
            .map(|t| {
                format!(
                    "<a href=\"?area={}&amp;tag={}\" data-area-tag=\"{}:{}\">#{}</a>",
                    area.slug(),
                    t.slug,
                    area.slug(),
                    t.slug,
                    t.slug
                )
            })
            .collect();
        out.push_str(&format!(
            "<li class=\"fe\"><a class=\"ft\" href=\"?area={}\" data-area=\"{}\"><span class=\"rn\">{}</span><span class=\"fn\">{}</span><span class=\"ld\"></span><span class=\"n\">{}</span></a><p class=\"fc\">{}</p></li>",
            area.slug(),
            area.slug(),
            area.numeral(),
            escape_xml(area.title_tr()),
            cnt,
            area_tags.join(", ")
        ));
    }
    out.push_str("</ol>");
    out.push_str(&format!(
        "<p class=\"hlinks\"><a href=\"?all=1\" data-all=\"1\">Tüm kayıtları tek listede gör ({}) →</a><a href=\"data/sample-email-tr.html\" target=\"_blank\" rel=\"noopener\">Günlük E-Posta Önizlemesi (10×3 = 30 Kayıt) ↗</a><a href=\"k/\">Düz metin dizini (k/)</a></p>",
        issue.items.len()
    ));
    out.push_str("</section>");
    out.push_str("</div>");
    out
}

fn inject_prerender_into_index_html(root_dir: &Path, issue: &DailyIssue) -> Result<(), String> {
    let index_path = root_dir.join("index.html");
    if !index_path.exists() {
        return Ok(());
    }
    let mut index_html = fs::read_to_string(&index_path).map_err(|e| e.to_string())?;

    // If style.css exists, inline it between <!-- INLINE_CSS_START --> and <!-- INLINE_CSS_END -->
    let css_path = root_dir.join("style.css");
    if css_path.exists() {
        if let Ok(css_raw) = fs::read_to_string(&css_path) {
            let start_marker = "<!-- INLINE_CSS_START -->";
            let end_marker = "<!-- INLINE_CSS_END -->";
            if let (Some(s_idx), Some(e_idx)) =
                (index_html.find(start_marker), index_html.find(end_marker))
            {
                if s_idx < e_idx {
                    let replacement =
                        format!("{}\n<style>{}</style>\n", start_marker, css_raw.trim());
                    index_html.replace_range(s_idx..e_idx, &replacement);
                }
            }
        }
    }

    // Inject pre-rendered `<div class="home" data-pre="1">` between <!-- PRERENDER_START --> and <!-- PRERENDER_END -->
    let pre_start = "<!-- PRERENDER_START -->";
    let pre_end = "<!-- PRERENDER_END -->";
    if let (Some(s_idx), Some(e_idx)) = (index_html.find(pre_start), index_html.find(pre_end)) {
        if s_idx < e_idx {
            let pre_html = render_prerendered_home_html(issue);
            let replacement = format!("{}\n{}\n", pre_start, pre_html);
            index_html.replace_range(s_idx..e_idx, &replacement);
        }
    }

    fs::write(&index_path, index_html).map_err(|e| e.to_string())?;
    Ok(())
}

fn update_archive_index(data_dir: &Path, issue: &DailyIssue) -> Result<(), String> {
    let archive_path = data_dir.join("archive.json");
    let mut entries: Vec<ArchiveEntry> = if archive_path.exists() {
        let raw = fs::read_to_string(&archive_path).unwrap_or_default();
        serde_json::from_str(&raw).unwrap_or_default()
    } else {
        Vec::new()
    };

    entries.retain(|e| e.date != issue.date);

    let top_headlines: Vec<String> = issue
        .items
        .iter()
        .filter(|i| i.is_top_pick)
        .take(3)
        .map(|i| i.headline.clone())
        .collect();

    entries.push(ArchiveEntry {
        issue_number: issue.issue_number,
        date: issue.date.clone(),
        edition_type: issue.edition_type,
        qualified_items: issue.stats.qualified_items,
        top_picks_count: issue.stats.top_picks_count,
        top_headlines,
    });

    entries.sort_by(|a, b| b.date.cmp(&a.date));

    let total = entries.len();
    for (idx, entry) in entries.iter_mut().enumerate() {
        entry.issue_number = (total - idx) as u32;
    }

    let json = serde_json::to_string_pretty(&entries).map_err(|e| e.to_string())?;
    fs::write(archive_path, json).map_err(|e| e.to_string())?;
    Ok(())
}

fn emit_tags_metadata(data_dir: &Path) -> Result<(), String> {
    let list: Vec<serde_json::Value> = CANON_TAGS
        .iter()
        .map(|t| {
            serde_json::json!({
                "slug": t.slug,
                "label": t.label,
                "area": t.area_slug
            })
        })
        .collect();
    let json = serde_json::to_string_pretty(&list).map_err(|e| e.to_string())?;
    fs::write(data_dir.join("tags.json"), json).map_err(|e| e.to_string())?;
    Ok(())
}

fn build_atom_feed(
    title: &str,
    subtitle: &str,
    base_url: &str,
    self_rel: &str,
    updated_iso: &str,
    items: &[NewsItem],
) -> String {
    let base = base_url.trim_end_matches('/');
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    xml.push_str("<feed xmlns=\"http://www.w3.org/2005/Atom\">\n");
    xml.push_str(&format!("  <title>{}</title>\n", escape_xml(title)));
    xml.push_str(&format!("  <subtitle>{}</subtitle>\n", escape_xml(subtitle)));
    xml.push_str(&format!(
        "  <link href=\"{}/{}\" rel=\"self\" type=\"application/atom+xml\"/>\n",
        escape_xml(base),
        escape_xml(self_rel)
    ));
    xml.push_str(&format!("  <link href=\"{}/\"/>\n", escape_xml(base)));
    xml.push_str(&format!(
        "  <id>{}/{}</id>\n",
        escape_xml(base),
        escape_xml(self_rel)
    ));
    xml.push_str(&format!("  <updated>{}</updated>\n", escape_xml(updated_iso)));

    for item in items.iter().take(40) {
        let item_detail_url = format!("{}/?item={}", base, item.id);
        xml.push_str("  <entry>\n");
        xml.push_str(&format!(
            "    <title>{}</title>\n",
            escape_xml(&item.headline)
        ));
        xml.push_str(&format!(
            "    <link href=\"{}\"/>\n",
            escape_xml(&item_detail_url)
        ));
        xml.push_str(&format!(
            "    <link rel=\"related\" href=\"{}\"/>\n",
            escape_xml(&item.url)
        ));
        xml.push_str(&format!(
            "    <id>{}/item/{}</id>\n",
            escape_xml(base),
            escape_xml(&item.id)
        ));
        xml.push_str(&format!(
            "    <updated>{}</updated>\n",
            escape_xml(&item.published_at)
        ));
        xml.push_str(&format!(
            "    <summary type=\"html\">{}</summary>\n",
            escape_xml(&format!(
                "<p>{}</p><p><strong>Source:</strong> <a href=\"{}\">{}</a> · <strong>Tags:</strong> {}</p>",
                escape_xml(&item.summary),
                escape_xml(&item.url),
                escape_xml(&item.source_name),
                item.tags
                    .iter()
                    .map(|t| format!("#{}", t))
                    .collect::<Vec<_>>()
                    .join(" ")
            ))
        ));
        xml.push_str("  </entry>\n");
    }

    xml.push_str("</feed>\n");
    xml
}

pub fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
