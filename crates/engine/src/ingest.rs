use ahbar_shared::{
    extract_arxiv_links, resolve_canonical_tag, Area, SourceDescriptor, SourceHealthRecord,
    SourceKind, SOURCES,
};
use chrono::Utc;
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct RawCandidate {
    #[allow(dead_code)]
    pub source_id: String,
    pub source_name: String,
    pub area: Area,
    pub is_paper: bool,
    pub authority_weight: f64,
    pub title: String,
    pub summary: String,
    pub url: String,
    pub authors: Vec<String>,
    pub pdf_url: Option<String>,
    pub html_ar5iv_url: Option<String>,
    pub code_url: Option<String>,
    pub discussion_url: Option<String>,
    pub discussion_label: Option<String>,
    pub community_points: u32,
    pub raw_tags: Vec<String>,
    pub published_at: String,
}

pub struct IngestResult {
    pub candidates: Vec<RawCandidate>,
    pub health: Vec<SourceHealthRecord>,
}

/// Fetches all configured primary sources in parallel using Tokio.
pub async fn fetch_all_sources() -> IngestResult {
    let client = Client::builder()
        .user_agent("AhbarBot/0.1 (+https://github.com/latifkedi/ahbar; technical-digest)")
        .timeout(Duration::from_secs(18))
        .build()
        .expect("Failed to initialize HTTP client");

    let mut handles = Vec::new();
    for src in SOURCES.iter().cloned() {
        let client_clone = client.clone();
        handles.push(tokio::spawn(async move {
            fetch_single_source(&client_clone, &src).await
        }));
    }

    let mut all_candidates = Vec::new();
    let mut health_records = Vec::new();

    for handle in handles {
        if let Ok((mut items, health)) = handle.await {
            all_candidates.append(&mut items);
            health_records.push(health);
        }
    }

    IngestResult {
        candidates: all_candidates,
        health: health_records,
    }
}

async fn fetch_single_source(
    client: &Client,
    src: &SourceDescriptor,
) -> (Vec<RawCandidate>, SourceHealthRecord) {
    let start = Instant::now();
    let now_iso = Utc::now().to_rfc3339();

    let resp = match client.get(src.url).send().await {
        Ok(r) => r,
        Err(e) => {
            return (
                Vec::new(),
                SourceHealthRecord {
                    id: src.id.to_string(),
                    name: src.name.to_string(),
                    area: src.area,
                    url: src.url.to_string(),
                    status: "error".to_string(),
                    items_fetched: 0,
                    latency_ms: start.elapsed().as_millis() as u64,
                    last_checked: now_iso,
                    error: Some(e.to_string()),
                },
            );
        }
    };

    if !resp.status().is_success() {
        let status_code = resp.status();
        return (
            Vec::new(),
            SourceHealthRecord {
                id: src.id.to_string(),
                name: src.name.to_string(),
                area: src.area,
                url: src.url.to_string(),
                status: format!("http_{}", status_code.as_u16()),
                items_fetched: 0,
                latency_ms: start.elapsed().as_millis() as u64,
                last_checked: now_iso,
                error: Some(format!("HTTP {}", status_code)),
            },
        );
    }

    let bytes = match resp.bytes().await {
        Ok(b) => b,
        Err(e) => {
            return (
                Vec::new(),
                SourceHealthRecord {
                    id: src.id.to_string(),
                    name: src.name.to_string(),
                    area: src.area,
                    url: src.url.to_string(),
                    status: "read_error".to_string(),
                    items_fetched: 0,
                    latency_ms: start.elapsed().as_millis() as u64,
                    last_checked: now_iso,
                    error: Some(e.to_string()),
                },
            );
        }
    };

    let parsed = match src.kind {
        SourceKind::ArxivApi | SourceKind::RssAtom => parse_rss_or_atom(&bytes, src),
        SourceKind::HackerNewsApi => parse_hn_algolia(&bytes, src),
        SourceKind::HuggingFacePapersApi => parse_hf_daily_papers(&bytes, src),
    };

    match parsed {
        Ok(items) => {
            let count = items.len();
            (
                items,
                SourceHealthRecord {
                    id: src.id.to_string(),
                    name: src.name.to_string(),
                    area: src.area,
                    url: src.url.to_string(),
                    status: "ok".to_string(),
                    items_fetched: count,
                    latency_ms: start.elapsed().as_millis() as u64,
                    last_checked: now_iso,
                    error: None,
                },
            )
        }
        Err(err_msg) => (
            Vec::new(),
            SourceHealthRecord {
                id: src.id.to_string(),
                name: src.name.to_string(),
                area: src.area,
                url: src.url.to_string(),
                status: "parse_error".to_string(),
                items_fetched: 0,
                latency_ms: start.elapsed().as_millis() as u64,
                last_checked: now_iso,
                error: Some(err_msg),
            },
        ),
    }
}

fn parse_rss_or_atom(bytes: &[u8], src: &SourceDescriptor) -> Result<Vec<RawCandidate>, String> {
    let feed = feed_rs::parser::parse(bytes).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    let now_iso = Utc::now().to_rfc3339();

    for entry in feed.entries.into_iter().take(35) {
        let raw_title = entry
            .title
            .map(|t| clean_html_text(&t.content))
            .unwrap_or_default();
        if raw_title.is_empty() {
            continue;
        }

        let raw_summary = entry
            .summary
            .map(|s| clean_html_text(&s.content))
            .or_else(|| {
                entry
                    .content
                    .and_then(|c| c.body)
                    .map(|b| clean_html_text(&b))
            })
            .unwrap_or_default();

        let primary_link = entry
            .links
            .iter()
            .find(|l| l.rel.as_deref() != Some("related"))
            .or_else(|| entry.links.first())
            .map(|l| l.href.clone())
            .unwrap_or_default();

        if primary_link.is_empty() {
            continue;
        }

        let mut pdf_url = entry
            .links
            .iter()
            .find(|l| {
                l.media_type.as_deref() == Some("application/pdf")
                    || l.href.ends_with(".pdf")
                    || l.href.contains("/pdf/")
            })
            .map(|l| l.href.clone());

        let mut html_ar5iv_url = None;
        let mut final_url = primary_link.clone();

        if let Some((abs, pdf, ar5iv)) = extract_arxiv_links(&primary_link) {
            final_url = abs;
            pdf_url = Some(pdf);
            html_ar5iv_url = Some(ar5iv);
        } else if primary_link.contains("eprint.iacr.org/") {
            let trimmed = primary_link.trim_end_matches('/');
            pdf_url = Some(format!("{}.pdf", trimmed));
        }

        let code_url = extract_github_repo_url(&raw_summary);

        let mut discussion_url = None;
        let mut discussion_label = None;
        if src.id == "lobsters" {
            // Lobste.rs puts the comments URL in entry.id or alternate link
            if entry.id.starts_with("https://lobste.rs/s/") {
                discussion_url = Some(entry.id.clone());
                discussion_label = Some("Lobsters".to_string());
            }
        }

        let authors: Vec<String> = entry
            .authors
            .into_iter()
            .map(|a| a.name.trim().to_string())
            .filter(|n| !n.is_empty())
            .take(6)
            .collect();

        let mut raw_tags: Vec<String> = src.default_tags.iter().map(|s| s.to_string()).collect();
        for cat in entry.categories {
            raw_tags.push(cat.term);
        }
        infer_tags_from_text(&raw_title, &raw_summary, &mut raw_tags);

        let published_at = entry
            .published
            .or(entry.updated)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| now_iso.clone());

        let area = infer_area_override(src.area, &raw_title, &raw_summary, &final_url, &raw_tags);

        out.push(RawCandidate {
            source_id: src.id.to_string(),
            source_name: src.name.to_string(),
            area,
            is_paper: src.is_academic_paper
                || final_url.contains("arxiv.org")
                || final_url.contains("eprint.iacr.org"),
            authority_weight: src.authority_weight,
            title: normalize_whitespace(&raw_title),
            summary: normalize_whitespace(&raw_summary),
            url: final_url,
            authors,
            pdf_url,
            html_ar5iv_url,
            code_url,
            discussion_url,
            discussion_label,
            community_points: if src.id == "lobsters" { 25 } else { 0 },
            raw_tags,
            published_at,
        });
    }

    Ok(out)
}

#[derive(Deserialize)]
struct HnAlgoliaResponse {
    hits: Vec<HnHit>,
}

#[derive(Deserialize)]
struct HnHit {
    #[serde(rename = "objectID")]
    object_id: String,
    title: Option<String>,
    url: Option<String>,
    points: Option<u32>,
    author: Option<String>,
    created_at: Option<String>,
}

fn parse_hn_algolia(bytes: &[u8], src: &SourceDescriptor) -> Result<Vec<RawCandidate>, String> {
    let parsed: HnAlgoliaResponse = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    let now_iso = Utc::now().to_rfc3339();

    for hit in parsed.hits {
        let title = match hit.title {
            Some(t) if !t.trim().is_empty() => t.trim().to_string(),
            _ => continue,
        };
        let url = match hit.url {
            Some(u) if u.starts_with("http") => u,
            _ => continue,
        };
        let points = hit.points.unwrap_or(10);
        if points < 15 {
            continue;
        }

        let hn_url = format!("https://news.ycombinator.com/item?id={}", hit.object_id);
        let mut pdf_url = None;
        let mut html_ar5iv_url = None;
        let mut final_url = url.clone();
        if let Some((abs, pdf, ar5iv)) = extract_arxiv_links(&url) {
            final_url = abs;
            pdf_url = Some(pdf);
            html_ar5iv_url = Some(ar5iv);
        }

        let code_url = if final_url.starts_with("https://github.com/") {
            Some(final_url.clone())
        } else {
            None
        };

        let mut raw_tags: Vec<String> = src.default_tags.iter().map(|s| s.to_string()).collect();
        infer_tags_from_text(&title, &final_url, &mut raw_tags);
        let area = infer_area_override(src.area, &title, "", &final_url, &raw_tags);

        out.push(RawCandidate {
            source_id: src.id.to_string(),
            source_name: src.name.to_string(),
            area,
            is_paper: final_url.contains("arxiv.org") || final_url.contains("eprint.iacr.org"),
            authority_weight: src.authority_weight,
            title: normalize_whitespace(&title),
            summary: String::new(),
            url: final_url,
            authors: hit.author.into_iter().collect(),
            pdf_url,
            html_ar5iv_url,
            code_url,
            discussion_url: Some(hn_url),
            discussion_label: Some(format!("HN ({} pts)", points)),
            community_points: points,
            raw_tags,
            published_at: hit.created_at.unwrap_or_else(|| now_iso.clone()),
        });
    }

    Ok(out)
}

#[derive(Deserialize)]
struct HfDailyPaperEntry {
    paper: HfPaperInner,
    #[serde(rename = "publishedAt")]
    published_at: Option<String>,
}

#[derive(Deserialize)]
struct HfPaperInner {
    id: String,
    title: Option<String>,
    summary: Option<String>,
    upvotes: Option<u32>,
    #[serde(default)]
    authors: Vec<HfAuthor>,
    #[serde(rename = "githubRepo")]
    github_repo: Option<String>,
}

#[derive(Deserialize)]
struct HfAuthor {
    name: Option<String>,
}

fn parse_hf_daily_papers(
    bytes: &[u8],
    src: &SourceDescriptor,
) -> Result<Vec<RawCandidate>, String> {
    let entries: Vec<HfDailyPaperEntry> =
        serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    let now_iso = Utc::now().to_rfc3339();

    for entry in entries.into_iter().take(25) {
        let p = entry.paper;
        let title = match p.title {
            Some(t) if !t.trim().is_empty() => normalize_whitespace(&t),
            _ => continue,
        };
        let summary = p
            .summary
            .map(|s| normalize_whitespace(&s))
            .unwrap_or_default();
        let paper_id = p.id.trim().to_string();
        let abs_url = format!("https://arxiv.org/abs/{}", paper_id);
        let pdf_url = Some(format!("https://arxiv.org/pdf/{}.pdf", paper_id));
        let html_ar5iv_url = Some(format!("https://ar5iv.labs.arxiv.org/html/{}", paper_id));
        let hf_discussion = format!("https://huggingface.co/papers/{}", paper_id);

        let upvotes = p.upvotes.unwrap_or(15);
        let code_url = p
            .github_repo
            .filter(|u| u.starts_with("http"))
            .or_else(|| extract_github_repo_url(&summary));

        let authors: Vec<String> = p
            .authors
            .into_iter()
            .filter_map(|a| a.name)
            .take(6)
            .collect();

        let mut raw_tags: Vec<String> = src.default_tags.iter().map(|s| s.to_string()).collect();
        infer_tags_from_text(&title, &summary, &mut raw_tags);

        out.push(RawCandidate {
            source_id: src.id.to_string(),
            source_name: src.name.to_string(),
            area: Area::Ai,
            is_paper: true,
            authority_weight: src.authority_weight,
            title,
            summary,
            url: abs_url,
            authors,
            pdf_url,
            html_ar5iv_url,
            code_url,
            discussion_url: Some(hf_discussion),
            discussion_label: Some(format!("HF Papers ({}▲)", upvotes)),
            community_points: upvotes * 3,
            raw_tags,
            published_at: entry.published_at.unwrap_or_else(|| now_iso.clone()),
        });
    }

    Ok(out)
}

pub fn clean_html_text(raw: &str) -> String {
    let fragment = scraper::Html::parse_fragment(raw);
    let mut text = String::new();
    for node in fragment.root_element().text() {
        text.push_str(node);
        text.push(' ');
    }
    normalize_whitespace(&text)
}

pub fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn extract_github_repo_url(text: &str) -> Option<String> {
    let re = Regex::new(r"https?://github\.com/[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+").ok()?;
    re.find(text).map(|m| {
        m.as_str()
            .trim_end_matches(|c: char| matches!(c, '.' | ',' | ')' | ']' | ';'))
            .to_string()
    })
}

pub fn infer_tags_from_text(title: &str, summary: &str, tags: &mut Vec<String>) {
    let combined = format!("{} {}", title, summary).to_ascii_lowercase();
    let keyword_map = [
        ("rust", "rust"),
        ("compiler", "compilers"),
        ("llvm", "compilers"),
        ("wasm", "typescript"),
        ("webassembly", "typescript"),
        ("postgres", "databases"),
        ("sqlite", "databases"),
        ("database", "databases"),
        ("distributed", "distributed-systems"),
        ("consensus", "consensus"),
        ("zero-knowledge", "zkp"),
        ("zk-snark", "zkp"),
        ("snark", "zkp"),
        ("post-quantum", "post-quantum"),
        ("lattice", "post-quantum"),
        ("smart contract", "smart-contracts"),
        ("ethereum", "consensus"),
        ("lean 4", "formal-proofs"),
        ("lean4", "formal-proofs"),
        ("coq", "formal-proofs"),
        ("rocq", "formal-proofs"),
        ("theorem", "logic"),
        ("prime", "number-theory"),
        ("graph", "combinatorics"),
        ("manifold", "topology"),
        ("pde", "analysis-pde"),
        ("qubit", "quantum-computing"),
        ("quantum error", "quantum-computing"),
        ("superconduct", "condensed-matter"),
        ("black hole", "astrophysics"),
        ("cosmolog", "astrophysics"),
        ("photon", "optics-photonics"),
        ("linux kernel", "kernel-os"),
        ("ebpf", "kernel-os"),
        ("risc-v", "semiconductors"),
        ("semiconductor", "semiconductors"),
        ("vulnerability", "infosec"),
        ("exploit", "infosec"),
        ("protein", "genomics-proteins"),
        ("genome", "genomics-proteins"),
        ("crispr", "genomics-proteins"),
        ("neural", "neuroscience"),
        ("fusion", "clean-tech"),
        ("battery", "materials-science"),
        ("quantization", "inference"),
        ("vllm", "inference"),
        ("agent", "agents"),
        ("retrieval", "rag"),
        ("diffusion", "computer-vision"),
        ("robot", "robotics"),
        ("reinforcement learning", "reinforcement-learning"),
    ];

    for (kw, canon) in keyword_map {
        if combined.contains(kw) {
            if let Some(resolved) = resolve_canonical_tag(canon) {
                tags.push(resolved.to_string());
            }
        }
    }
}

fn infer_area_override(
    default_area: Area,
    title: &str,
    summary: &str,
    url: &str,
    raw_tags: &[String],
) -> Area {
    if default_area != Area::Cs {
        return default_area;
    }
    let combined = format!("{} {} {}", title, summary, url).to_ascii_lowercase();
    if combined.contains("zero-knowledge")
        || combined.contains("zk-snark")
        || combined.contains("eprint.iacr.org")
        || combined.contains("ethereum")
        || combined.contains("cryptograph")
    {
        return Area::Crypto;
    }
    if combined.contains("qubit")
        || combined.contains("quantum")
        || combined.contains("superconduct")
        || combined.contains("astrophys")
    {
        return Area::Physics;
    }
    if combined.contains("lean 4")
        || combined.contains("theorem")
        || combined.contains("conjecture")
        || combined.contains("number theory")
    {
        return Area::Math;
    }
    if combined.contains("llm")
        || combined.contains("transformer")
        || combined.contains("language model")
        || combined.contains("huggingface.co")
    {
        return Area::Ai;
    }
    if combined.contains("linux kernel")
        || combined.contains("cve-")
        || combined.contains("zero-day")
        || combined.contains("risc-v")
        || combined.contains("semiconductor")
    {
        return Area::Systems;
    }
    if combined.contains("CRISPR")
        || combined.contains("protein folding")
        || combined.contains("genomic")
        || combined.contains("bioinformatics")
    {
        return Area::Bio;
    }
    let _ = raw_tags;
    default_area
}
