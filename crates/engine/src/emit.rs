use ahbar_shared::{
    ArchiveEntry, Area, DailyIssue, NewsItem, SourceHealthRecord, CANON_TAGS,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Writes the chunked static JSON payloads (`data/latest.json`, `data/issues/<date>.json`,
/// `data/archive.json`, `data/health.json`, `data/tags.json`) and generates static Atom 1.0
/// feeds (`feed/all.xml`, `feed/area-<slug>.xml`, `feed/tag-<slug>.xml`).
pub fn emit_all(
    root_dir: &Path,
    issue: &DailyIssue,
    health: &[SourceHealthRecord],
    site_base_url: &str,
) -> Result<(), String> {
    let data_dir = root_dir.join("data");
    let issues_dir = data_dir.join("issues");
    let feed_dir = root_dir.join("feed");

    fs::create_dir_all(&issues_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&feed_dir).map_err(|e| e.to_string())?;

    // 1. Write data/latest.json
    let latest_json = serde_json::to_string_pretty(issue).map_err(|e| e.to_string())?;
    fs::write(data_dir.join("latest.json"), &latest_json).map_err(|e| e.to_string())?;

    // 2. Write data/issues/YYYY-MM-DD.json
    let issue_path = issues_dir.join(format!("{}.json", issue.date));
    fs::write(&issue_path, &latest_json).map_err(|e| e.to_string())?;

    // 3. Update data/archive.json
    update_archive_index(&data_dir, issue)?;

    // 4. Write data/health.json
    let health_json = serde_json::to_string_pretty(health).map_err(|e| e.to_string())?;
    fs::write(data_dir.join("health.json"), health_json).map_err(|e| e.to_string())?;

    // 5. Write data/tags.json (canonical tags metadata for frontend filter chips)
    emit_tags_metadata(&data_dir)?;

    // 6. Emit Atom 1.0 XML feeds (root feed.xml + feed/all.xml + per-area + per-tag)
    let all_atom = build_atom_feed(
        "Ahbar — All Technical & Scientific Dispatches",
        "Daily curated dispatches across computer science, AI, mathematics, physics, systems, bio and cryptography.",
        site_base_url,
        "feed/all.xml",
        &issue.generated_at,
        &issue.items,
    );
    fs::write(feed_dir.join("all.xml"), &all_atom).map_err(|e| e.to_string())?;
    fs::write(root_dir.join("feed.xml"), &all_atom).map_err(|e| e.to_string())?;

    // Per-area feeds
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

    // Re-number sequentially from oldest (1) to newest (N)
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
    xml.push_str(&format!("  <id>{}/{}</id>\n", escape_xml(base), escape_xml(self_rel)));
    xml.push_str(&format!("  <updated>{}</updated>\n", escape_xml(updated_iso)));

    for item in items.iter().take(40) {
        let item_detail_url = format!("{}/?item={}", base, item.id);
        xml.push_str("  <entry>\n");
        xml.push_str(&format!("    <title>{}</title>\n", escape_xml(&item.headline)));
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
                item.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" ")
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
