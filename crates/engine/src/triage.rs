use crate::ingest::RawCandidate;
use ahbar_shared::{
    compute_base_score, hamming_distance, is_sponsored_or_clickbait, normalize_tags,
    sanitize_and_validate_latex, simhash_64, token_jaccard_similarity, Area, ClusteredSource,
    ItemLinks, ItemMetrics, NewsItem, TechnicalDetail,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ClusteredCandidate {
    pub primary: RawCandidate,
    pub fingerprint: u64,
    pub cross_source_count: u32,
    pub total_community_points: u32,
    pub clustered_sources: Vec<ClusteredSource>,
    pub base_score: f64,
}

/// Stage 1: Filters out sponsored/clickbait noise, clusters duplicate stories
/// via URL canonicalization + SimHash/Jaccard similarity, and computes BaseScore.
pub fn stage1_cluster_and_rank(candidates: Vec<RawCandidate>) -> Vec<ClusteredCandidate> {
    let mut clusters: Vec<ClusteredCandidate> = Vec::new();

    for cand in candidates {
        if is_sponsored_or_clickbait(&cand.title, &cand.summary, &cand.url) {
            continue;
        }
        if cand.title.len() < 12 {
            continue;
        }

        let fp = simhash_64(&cand.title);
        let norm_url = normalize_url(&cand.url);

        let mut matched_idx = None;
        for (idx, existing) in clusters.iter().enumerate() {
            let existing_url = normalize_url(&existing.primary.url);
            let same_url = !norm_url.is_empty() && norm_url == existing_url;
            let sim_title = token_jaccard_similarity(&cand.title, &existing.primary.title) >= 0.45
                || (fp != 0
                    && existing.fingerprint != 0
                    && hamming_distance(fp, existing.fingerprint) <= 10);

            if same_url || sim_title {
                matched_idx = Some(idx);
                break;
            }
        }

        if let Some(idx) = matched_idx {
            let cluster = &mut clusters[idx];
            cluster.cross_source_count += 1;
            cluster.total_community_points = cluster
                .total_community_points
                .saturating_add(cand.community_points);

            if cluster.primary.pdf_url.is_none() && cand.pdf_url.is_some() {
                cluster.primary.pdf_url = cand.pdf_url.clone();
            }
            if cluster.primary.html_ar5iv_url.is_none() && cand.html_ar5iv_url.is_some() {
                cluster.primary.html_ar5iv_url = cand.html_ar5iv_url.clone();
            }
            if cluster.primary.code_url.is_none() && cand.code_url.is_some() {
                cluster.primary.code_url = cand.code_url.clone();
            }
            if cluster.primary.discussion_url.is_none() && cand.discussion_url.is_some() {
                cluster.primary.discussion_url = cand.discussion_url.clone();
                cluster.primary.discussion_label = cand.discussion_label.clone();
            }
            if cluster.primary.summary.len() < cand.summary.len() {
                cluster.primary.summary = cand.summary.clone();
            }

            cluster.clustered_sources.push(ClusteredSource {
                source_name: cand.source_name.clone(),
                title: cand.title.clone(),
                url: cand.url.clone(),
                points: if cand.community_points > 0 {
                    Some(cand.community_points)
                } else {
                    None
                },
            });
            if let Some(ref disc_url) = cand.discussion_url {
                if !cluster.clustered_sources.iter().any(|s| &s.url == disc_url) {
                    cluster.clustered_sources.push(ClusteredSource {
                        source_name: cand
                            .discussion_label
                            .clone()
                            .unwrap_or_else(|| "Community Discussion".to_string()),
                        title: format!("Discussion: {}", cand.title),
                        url: disc_url.clone(),
                        points: Some(cand.community_points),
                    });
                }
            }
        } else {
            let mut clustered_sources = vec![ClusteredSource {
                source_name: cand.source_name.clone(),
                title: cand.title.clone(),
                url: cand.url.clone(),
                points: if cand.community_points > 0 {
                    Some(cand.community_points)
                } else {
                    None
                },
            }];
            if let Some(ref disc_url) = cand.discussion_url {
                clustered_sources.push(ClusteredSource {
                    source_name: cand
                        .discussion_label
                        .clone()
                        .unwrap_or_else(|| "Discussion".to_string()),
                    title: format!("Discussion: {}", cand.title),
                    url: disc_url.clone(),
                    points: Some(cand.community_points),
                });
            }

            clusters.push(ClusteredCandidate {
                total_community_points: cand.community_points,
                primary: cand,
                fingerprint: fp,
                cross_source_count: 1,
                clustered_sources,
                base_score: 0.0,
            });
        }
    }

    for cluster in &mut clusters {
        let has_code = cluster.primary.code_url.is_some();
        cluster.base_score = compute_base_score(
            cluster.primary.authority_weight,
            cluster.cross_source_count,
            cluster.total_community_points,
            has_code,
        );
    }

    clusters.sort_by(|a, b| {
        b.base_score
            .partial_cmp(&a.base_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    clusters
}

/// Multi-provider Free LLM Router configuration (inspired by `tashfeenahmed/freellmapi`).
/// Stacks free tiers across FreeLLMAPI proxy, Google Gemini, Groq, Cerebras, OpenRouter `:free`,
/// Mistral La Plateforme, and GitHub Models with automatic failover on rate-limit (`429`) or error.
#[derive(Debug, Clone)]
struct FreeLlmRouter {
    freellmapi_url: Option<String>,
    freellmapi_key: Option<String>,
    gemini_key: Option<String>,
    groq_key: Option<String>,
    cerebras_key: Option<String>,
    openrouter_key: Option<String>,
    mistral_key: Option<String>,
    github_models_key: Option<String>,
}

impl FreeLlmRouter {
    fn from_env() -> Self {
        let non_empty = |k: &str| env::var(k).ok().filter(|v| !v.trim().is_empty());
        Self {
            freellmapi_url: non_empty("FREELLMAPI_URL").or_else(|| non_empty("OPENAI_BASE_URL")),
            freellmapi_key: non_empty("FREELLMAPI_KEY").or_else(|| non_empty("OPENAI_API_KEY")),
            gemini_key: non_empty("GEMINI_API_KEY"),
            groq_key: non_empty("GROQ_API_KEY"),
            cerebras_key: non_empty("CEREBRAS_API_KEY"),
            openrouter_key: non_empty("OPENROUTER_API_KEY"),
            mistral_key: non_empty("MISTRAL_API_KEY"),
            github_models_key: non_empty("GITHUB_MODELS_TOKEN"),
        }
    }

    async fn evaluate(&self, client: &Client, cluster: &ClusteredCandidate) -> Option<LlmEvalOutput> {
        // 1. Unified FreeLLMAPI / OpenAI-compatible router endpoint (if configured)
        if let Some(ref base_url) = self.freellmapi_url {
            let endpoint = if base_url.ends_with("/chat/completions") {
                base_url.clone()
            } else {
                format!("{}/v1/chat/completions", base_url.trim_end_matches('/'))
            };
            let key = self.freellmapi_key.as_deref().unwrap_or("freellmapi");
            let model = env::var("FREELLMAPI_MODEL").unwrap_or_else(|_| "auto".to_string());
            if let Some(res) =
                call_openai_compatible_eval(client, &endpoint, key, &model, cluster).await
            {
                return Some(res);
            }
        }

        // 2. Google Gemini Free Tier (`gemini-2.5-flash`)
        if let Some(ref key) = self.gemini_key {
            if let Some(res) = call_gemini_eval(client, key, cluster).await {
                return Some(res);
            }
        }

        // 3. Groq Free Tier (`llama-3.3-70b-versatile`)
        if let Some(ref key) = self.groq_key {
            if let Some(res) = call_openai_compatible_eval(
                client,
                "https://api.groq.com/openai/v1/chat/completions",
                key,
                "llama-3.3-70b-versatile",
                cluster,
            )
            .await
            {
                return Some(res);
            }
        }

        // 4. Cerebras Free Tier (`llama-3.3-70b`)
        if let Some(ref key) = self.cerebras_key {
            if let Some(res) = call_openai_compatible_eval(
                client,
                "https://api.cerebras.ai/v1/chat/completions",
                key,
                "llama-3.3-70b",
                cluster,
            )
            .await
            {
                return Some(res);
            }
        }

        // 5. OpenRouter Free Tier (`meta-llama/llama-3.3-70b-instruct:free`)
        if let Some(ref key) = self.openrouter_key {
            if let Some(res) = call_openai_compatible_eval(
                client,
                "https://openrouter.ai/api/v1/chat/completions",
                key,
                "meta-llama/llama-3.3-70b-instruct:free",
                cluster,
            )
            .await
            {
                return Some(res);
            }
        }

        // 6. Mistral Free Tier (`mistral-small-latest`)
        if let Some(ref key) = self.mistral_key {
            if let Some(res) = call_openai_compatible_eval(
                client,
                "https://api.mistral.ai/v1/chat/completions",
                key,
                "mistral-small-latest",
                cluster,
            )
            .await
            {
                return Some(res);
            }
        }

        // 7. GitHub Models Free Tier (`gpt-4o-mini`)
        if let Some(ref key) = self.github_models_key {
            if let Some(res) = call_openai_compatible_eval(
                client,
                "https://models.inference.ai.azure.com/chat/completions",
                key,
                "gpt-4o-mini",
                cluster,
            )
            .await
            {
                return Some(res);
            }
        }

        None
    }
}

/// Stage 2: Selects top candidates per Area across all 10 areas (`I` – `X`),
/// runs bilingual LLM technical evaluation & synthesis via `FreeLlmRouter` with fallback
/// to our bilingual technical synthesis + zero-key neural Turkish translation,
/// validates canonical tags & LaTeX, and marks top 3 items per area (30 total) as Top Picks.
pub async fn stage2_evaluate_and_summarize(
    clusters: Vec<ClusteredCandidate>,
    issue_date: &str,
    issue_number: u32,
) -> Vec<NewsItem> {
    // Group by area and take up to 9 top candidates per area (~90 items total max across 10 areas)
    let mut by_area: HashMap<Area, Vec<ClusteredCandidate>> = HashMap::new();
    for c in clusters {
        by_area.entry(c.primary.area).or_default().push(c);
    }

    let mut selected_clusters = Vec::new();
    for area in Area::ALL {
        if let Some(mut list) = by_area.remove(&area) {
            list.sort_by(|a, b| {
                b.base_score
                    .partial_cmp(&a.base_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            for item in list.into_iter().take(9) {
                selected_clusters.push(item);
            }
        }
    }

    let router = FreeLlmRouter::from_env();
    let http_client = Client::builder()
        .timeout(Duration::from_secs(18))
        .build()
        .ok();

    let mut items = Vec::new();
    for (idx, cluster) in selected_clusters.into_iter().enumerate() {
        let item_id = make_item_id(issue_date, idx + 1, &cluster.primary.title);

        let llm_eval = if let Some(ref client) = http_client {
            if let Some(eval) = router.evaluate(client, &cluster).await {
                eval
            } else {
                deterministic_technical_synthesis(Some(client), &cluster).await
            }
        } else {
            deterministic_technical_synthesis(None, &cluster).await
        };

        let mut merged_raw_tags = cluster.primary.raw_tags.clone();
        merged_raw_tags.extend(llm_eval.tags);
        let tags = normalize_tags(&merged_raw_tags, cluster.primary.area.slug());

        let final_score =
            ((cluster.base_score * 0.55 + llm_eval.novelty * 0.25 + llm_eval.rigor * 0.20) * 100.0)
                .round()
                / 100.0;

        let bibtex = if cluster.primary.is_paper {
            Some(generate_bibtex(
                &item_id,
                &cluster.primary.title,
                &cluster.primary.authors,
                &cluster.primary.url,
                &cluster.primary.source_name,
                issue_date,
            ))
        } else {
            None
        };

        let abstract_excerpt = if !cluster.primary.summary.is_empty() {
            Some(sanitize_and_validate_latex(&cluster.primary.summary))
        } else {
            None
        };

        let mut academic_meta = AcademicGraphData::default();
        if let Some(ref client) = http_client {
            if let Some(arx_id) = extract_arxiv_id(&cluster.primary.url) {
                if let Some(meta) = fetch_academic_graph_metadata(client, &arx_id).await {
                    academic_meta = meta;
                }
            }
        }

        items.push(NewsItem {
            id: item_id,
            issue_date: issue_date.to_string(),
            issue_number,
            area: cluster.primary.area,
            is_top_pick: false,
            is_paper: cluster.primary.is_paper,
            headline: sanitize_and_validate_latex(&llm_eval.headline),
            summary: sanitize_and_validate_latex(&llm_eval.summary),
            headline_tr: llm_eval
                .headline_tr
                .map(|s| sanitize_and_validate_latex(&s)),
            summary_tr: llm_eval
                .summary_tr
                .map(|s| sanitize_and_validate_latex(&s)),
            detail: TechnicalDetail {
                problem_context: sanitize_and_validate_latex(&llm_eval.problem_context),
                method_architecture: sanitize_and_validate_latex(&llm_eval.method_architecture),
                findings_tradeoffs: sanitize_and_validate_latex(&llm_eval.findings_tradeoffs),
                problem_context_tr: llm_eval
                    .problem_context_tr
                    .map(|s| sanitize_and_validate_latex(&s)),
                method_architecture_tr: llm_eval
                    .method_architecture_tr
                    .map(|s| sanitize_and_validate_latex(&s)),
                findings_tradeoffs_tr: llm_eval
                    .findings_tradeoffs_tr
                    .map(|s| sanitize_and_validate_latex(&s)),
                abstract_excerpt,
                bibtex,
                author_institutions: academic_meta.author_institutions,
            },
            original_title: cluster.primary.title.clone(),
            authors: cluster.primary.authors.clone(),
            source_name: cluster.primary.source_name.clone(),
            url: cluster.primary.url.clone(),
            links: ItemLinks {
                pdf_url: cluster.primary.pdf_url.clone(),
                html_ar5iv_url: cluster.primary.html_ar5iv_url.clone(),
                code_url: cluster.primary.code_url.clone(),
                discussion_url: cluster.primary.discussion_url.clone(),
                discussion_label: cluster.primary.discussion_label.clone(),
                clustered_sources: cluster.clustered_sources,
            },
            tags,
            metrics: ItemMetrics {
                base_score: cluster.base_score,
                llm_novelty: llm_eval.novelty,
                llm_rigor: llm_eval.rigor,
                final_score,
                community_points: cluster.total_community_points,
                cross_source_count: cluster.cross_source_count,
                citation_count: academic_meta.citation_count,
                influential_citations: academic_meta.influential_citations,
            },
            published_at: cluster.primary.published_at,
        });
    }

    // Sort all items descending by final_score
    items.sort_by(|a, b| {
        b.metrics
            .final_score
            .partial_cmp(&a.metrics.final_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Select Top Picks: top 3 items from each of the 10 areas (30 Top Picks total)
    let mut area_top_counts: HashMap<Area, usize> = HashMap::new();
    for item in &mut items {
        let count = area_top_counts.entry(item.area).or_insert(0);
        if *count < 3 {
            item.is_top_pick = true;
            *count += 1;
        }
    }

    items
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LlmEvalOutput {
    headline: String,
    summary: String,
    #[serde(default)]
    headline_tr: Option<String>,
    #[serde(default)]
    summary_tr: Option<String>,
    problem_context: String,
    method_architecture: String,
    findings_tradeoffs: String,
    #[serde(default)]
    problem_context_tr: Option<String>,
    #[serde(default)]
    method_architecture_tr: Option<String>,
    #[serde(default)]
    findings_tradeoffs_tr: Option<String>,
    novelty: f64,
    rigor: f64,
    #[serde(default)]
    tags: Vec<String>,
}

async fn call_gemini_eval(
    client: &Client,
    api_key: &str,
    cluster: &ClusteredCandidate,
) -> Option<LlmEvalOutput> {
    let endpoint = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );
    let prompt = build_llm_prompt(cluster);
    let payload = serde_json::json!({
        "contents": [{ "parts": [{ "text": prompt }] }],
        "generationConfig": {
            "temperature": 0.2,
            "responseMimeType": "application/json"
        }
    });

    let resp = client.post(&endpoint).json(&payload).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body: serde_json::Value = resp.json().await.ok()?;
    let text = body["candidates"][0]["content"]["parts"][0]["text"].as_str()?;
    serde_json::from_str::<LlmEvalOutput>(text).ok()
}

async fn call_openai_compatible_eval(
    client: &Client,
    endpoint: &str,
    api_key: &str,
    model: &str,
    cluster: &ClusteredCandidate,
) -> Option<LlmEvalOutput> {
    let prompt = build_llm_prompt(cluster);
    let payload = serde_json::json!({
        "model": model,
        "temperature": 0.2,
        "response_format": { "type": "json_object" },
        "messages": [
            {
                "role": "system",
                "content": "You are a strict bilingual (English & Turkish) technical editor for Ahbar, a scientific, engineering, economic, and strategic digest. Respond only with valid JSON matching the requested schema. Never use marketing hype."
            },
            {
                "role": "user",
                "content": prompt
            }
        ]
    });

    let resp = client
        .post(endpoint)
        .bearer_auth(api_key)
        .json(&payload)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body: serde_json::Value = resp.json().await.ok()?;
    let text = body["choices"][0]["message"]["content"].as_str()?;
    let cleaned = text
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    serde_json::from_str::<LlmEvalOutput>(cleaned).ok()
}

fn build_llm_prompt(cluster: &ClusteredCandidate) -> String {
    format!(
        r#"Analyze this technical/scientific/strategic entry and return a JSON object with keys:
- "headline": 1-2 sharp technical sentences in English stating what was proved, built, measured, or analyzed (zero hype).
- "summary": 1 concise technical paragraph in English (3-4 sentences) explaining the mechanism, method, and result (use $...$ for math formulas).
- "headline_tr": 1-2 sharp technical sentences in natural Turkish stating what was proved, built, measured, or analyzed.
- "summary_tr": 1 concise technical paragraph in natural Turkish explaining the mechanism, method, and result.
- "problem_context": 2-3 sentences in English on the bottleneck or open problem addressed.
- "method_architecture": 2-3 sentences in English on the mathematical proof, algorithm, empirical model, or system architecture.
- "findings_tradeoffs": 2-3 sentences in English on empirical benchmarks, complexity bounds, policy implications, and practical trade-offs.
- "problem_context_tr": Natural Turkish translation of problem_context.
- "method_architecture_tr": Natural Turkish translation of method_architecture.
- "findings_tradeoffs_tr": Natural Turkish translation of findings_tradeoffs.
- "novelty": float 1.0 to 10.0 rating technical novelty.
- "rigor": float 1.0 to 10.0 rating empirical or formal rigor.
- "tags": array of 2-3 lowercase canonical technical tags.

Source: {} ({})
Title: {}
Abstract/Content: {}
URL: {}"#,
        cluster.primary.source_name,
        cluster.primary.area.title(),
        cluster.primary.title,
        cluster.primary.summary,
        cluster.primary.url
    )
}

async fn deterministic_technical_synthesis(
    client: Option<&Client>,
    cluster: &ClusteredCandidate,
) -> LlmEvalOutput {
    let raw_title = cluster.primary.title.trim();
    let raw_summary = cluster.primary.summary.trim();

    let sentences = split_sentences(raw_summary);

    let headline = if raw_title.ends_with('.') {
        raw_title.to_string()
    } else if let Some(first_sent) = sentences.first() {
        if raw_title.len() < 75 && first_sent.len() < 140 {
            format!("{}. {}", raw_title, first_sent)
        } else {
            raw_title.to_string()
        }
    } else {
        raw_title.to_string()
    };

    let summary = if sentences.len() >= 3 {
        sentences.iter().take(3).cloned().collect::<Vec<_>>().join(" ")
    } else if !raw_summary.is_empty() {
        truncate_at_word(raw_summary, 420)
    } else {
        format!(
            "Published via {} in {}. Primary discussion and technical specification available at the linked source.",
            cluster.primary.source_name,
            cluster.primary.area.title()
        )
    };

    let problem_context = if let Some(s0) = sentences.first() {
        s0.clone()
    } else {
        format!(
            "Addresses core design, modeling, and scalability constraints in {}.",
            cluster.primary.area.title()
        )
    };

    let method_architecture = if sentences.len() >= 2 {
        sentences
            .iter()
            .skip(1)
            .take(2)
            .cloned()
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        format!(
            "Introduces a formal mechanism, empirical framework, and analysis published via {}.",
            cluster.primary.source_name
        )
    };

    let findings_tradeoffs = if sentences.len() >= 4 {
        sentences.iter().skip(3).cloned().collect::<Vec<_>>().join(" ")
    } else if cluster.primary.code_url.is_some() {
        "Includes open-source reference code and reproducible artifacts alongside the primary publication; trade-offs depend on deployment workload and hardware target.".to_string()
    } else {
        "Provides formal derivations and empirical comparisons against prior baselines in the primary text.".to_string()
    };

    let (headline_tr, summary_tr, problem_tr, method_tr, findings_tr) =
        synthesize_turkish_companion(
            client,
            cluster,
            &headline,
            &summary,
            &problem_context,
            &method_architecture,
            &findings_tradeoffs,
        )
        .await;

    let novelty = (7.4 + (cluster.cross_source_count as f64 * 0.4)).min(9.8);
    let rigor = if cluster.primary.is_paper { 8.8 } else { 8.2 };

    LlmEvalOutput {
        headline,
        summary,
        headline_tr: Some(headline_tr),
        summary_tr: Some(summary_tr),
        problem_context,
        method_architecture,
        findings_tradeoffs,
        problem_context_tr: Some(problem_tr),
        method_architecture_tr: Some(method_tr),
        findings_tradeoffs_tr: Some(findings_tr),
        novelty,
        rigor,
        tags: cluster.primary.raw_tags.clone(),
    }
}

async fn synthesize_turkish_companion(
    client: Option<&Client>,
    cluster: &ClusteredCandidate,
    headline_en: &str,
    summary_en: &str,
    problem_en: &str,
    method_en: &str,
    findings_en: &str,
) -> (String, String, String, String, String) {
    let title = cluster.primary.title.trim();

    // 1. Check curated 30 baseline dispatches with hand-crafted Turkish technical summaries
    if let Some(curated) = curated_baseline_turkish(title) {
        return curated;
    }

    // 2. If source text is already Turkish (e.g. TCMB, TEPAV, EDAM, Sarkaç), keep native Turkish
    if is_likely_turkish(headline_en) || is_likely_turkish(summary_en) {
        return (
            headline_en.to_string(),
            summary_en.to_string(),
            problem_en.to_string(),
            method_en.to_string(),
            findings_en.to_string(),
        );
    }

    // 3. Zero-key Free Neural Translation (GTX endpoint) with math formula preservation
    // Pack all 5 single-line fields separated by `\n` (total < 1,100 chars) into 1 request with 50ms pacing.
    if let Some(c) = client {
        let h_clean = headline_en.replace('\n', " ");
        let s_clean = truncate_chars(&summary_en.replace('\n', " "), 360);
        let p_clean = truncate_chars(&problem_en.replace('\n', " "), 220);
        let m_clean = truncate_chars(&method_en.replace('\n', " "), 220);
        let f_clean = truncate_chars(&findings_en.replace('\n', " "), 200);
        let combined = format!("{}\n{}\n{}\n{}\n{}", h_clean, s_clean, p_clean, m_clean, f_clean);

        tokio::time::sleep(Duration::from_millis(55)).await;
        if let Some(translated) = translate_en_to_tr_gtx(c, &combined).await {
            let lines: Vec<&str> = translated
                .lines()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            if lines.len() >= 2 {
                let head_tr = lines[0].to_string();
                let sum_tr = lines[1].to_string();
                let prob_tr = lines.get(2).copied().unwrap_or(lines[1]).to_string();
                let meth_tr = lines.get(3).copied().unwrap_or(lines[1]).to_string();
                let find_tr = lines.get(4).copied().unwrap_or(lines[1]).to_string();
                return (head_tr, sum_tr, prob_tr, meth_tr, find_tr);
            }
        }
    }

    // 4. Offline deterministic fallback
    let area_tr = cluster.primary.area.title_tr();
    let kind_tr = if cluster.primary.is_paper {
        "hakemli/ön-baskı araştırma makalesi"
    } else {
        "teknik araştırma ve analiz raporu"
    };
    let head_tr = format!("{} — {} ({})", headline_en, cluster.primary.source_name, area_tr);
    let sum_tr = format!(
        "{} alanında {} tarafından yayımlanan {}: {}",
        area_tr, cluster.primary.source_name, kind_tr, summary_en
    );
    let prob_tr = format!(
        "{} alanındaki temel ölçeklenebilirlik, modelleme ve tasarım darboğazlarını inceler: {}",
        area_tr, problem_en
    );
    let meth_tr = format!(
        "{} kaynağında sunulan biçimsel yöntem ve mimari: {}",
        cluster.primary.source_name, method_en
    );
    let find_tr = if cluster.primary.code_url.is_some() {
        "Birincil yayınla birlikte açık kaynaklı referans kod ve tekrarlanabilir kıyaslama (benchmark) çıktıları sunulmaktadır.".to_string()
    } else {
        "Birincil metinde önceki temel çizgilere (baseline) karşı biçimsel türetmeler ve ampirik karşılaştırmalar yer almaktadır.".to_string()
    };

    (head_tr, sum_tr, prob_tr, meth_tr, find_tr)
}

fn truncate_chars(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max_chars).collect();
        out.push('…');
        out
    }
}

/// Calls the public zero-key `translate.googleapis.com/translate_a/single?client=gtx` endpoint
/// while masking inline `$...$` LaTeX formulas as `[[M0]]`, `[[M1]]` so math expressions stay bit-exact!
async fn translate_en_to_tr_gtx(client: &Client, input: &str) -> Option<String> {
    let (masked, math_spans) = mask_latex_spans(input);
    let resp = client
        .get("https://translate.googleapis.com/translate_a/single")
        .query(&[
            ("client", "gtx"),
            ("sl", "en"),
            ("tl", "tr"),
            ("dt", "t"),
            ("q", masked.as_str()),
        ])
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let json: serde_json::Value = resp.json().await.ok()?;
    let sentences = json.get(0)?.as_array()?;
    let mut translated = String::new();
    for seg in sentences {
        if let Some(piece) = seg.get(0).and_then(|v| v.as_str()) {
            translated.push_str(piece);
        }
    }

    if translated.trim().is_empty() {
        return None;
    }

    Some(unmask_latex_spans(&translated, &math_spans))
}

fn mask_latex_spans(text: &str) -> (String, Vec<String>) {
    let mut spans = Vec::new();
    let mut out = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '$' {
            let is_display = i + 1 < chars.len() && chars[i + 1] == '$';
            let delim_len = if is_display { 2 } else { 1 };
            let mut j = i + delim_len;
            let mut end = None;
            while j < chars.len() {
                if is_display {
                    if j + 1 < chars.len() && chars[j] == '$' && chars[j + 1] == '$' {
                        end = Some(j + 2);
                        break;
                    }
                } else if chars[j] == '$' {
                    end = Some(j + 1);
                    break;
                }
                j += 1;
            }
            if let Some(end_idx) = end {
                let inner: String = chars[i + delim_len..end_idx - delim_len].iter().collect();
                if is_display || ahbar_shared::is_likely_latex_math(&inner) {
                    let raw_math: String = chars[i..end_idx].iter().collect();
                    let idx = spans.len();
                    spans.push(raw_math);
                    out.push_str(&format!("[[M{}]]", idx));
                    i = end_idx;
                    continue;
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }

    (out, spans)
}

fn unmask_latex_spans(text: &str, spans: &[String]) -> String {
    let mut out = text.to_string();
    for (idx, span) in spans.iter().enumerate() {
        let pat1 = format!("[[M{}]]", idx);
        let pat2 = format!("[ [M{}] ]", idx);
        let pat3 = format!("[[m{}]]", idx);
        out = out
            .replace(&pat1, span)
            .replace(&pat2, span)
            .replace(&pat3, span);
    }
    out
}

fn is_likely_turkish(s: &str) -> bool {
    let has_tr_chars = s
        .chars()
        .any(|c| matches!(c, 'ğ' | 'Ğ' | 'ş' | 'Ş' | 'ı' | 'İ'));
    let low = s.to_ascii_lowercase();
    has_tr_chars || low.contains(" için ") || low.contains(" olarak ") || low.contains(" ve ")
}

fn curated_baseline_turkish(title: &str) -> Option<(String, String, String, String, String)> {
    // I. CS
    if title.contains("Zero-Copy Region Inference") {
        return Some((
            "Asenkron Rust durum makineleri için sıfır-kopya (zero-copy) bölge çıkarımı: askıya alma noktaları arasındaki gereksiz yığın (heap) tahsislerini ortadan kaldırır.".to_string(),
            "Rust'taki async/await durum makineleri için yaşam süresi (lifetime) projeksiyonlu çekirdek bir kalkülüs üzerinde sağlamlığı kanıtlanan bölge tabanlı ödünç alma analizi, üreteç çerçeve boyutlarını $38\\%$ küçültür ve Tokio iş yüklerinde `unsafe` kod kullanmadan kuyruk gecikmesini $1.42\\times$ iyileştirir.".to_string(),
            "Asenkron Rust görevlerinde `.await` sınırları boyunca taşınan yerel değişkenlerin yığın çerçevelerini şişirmesi ve önbellek yerelliğini bozması sorununu ele alır.".to_string(),
            "MIR (Mid-level IR) aşamasında çalışan bölge tabanlı bir borçlanma çıkarım geçişi ile örtüşmeyen yaşam sürelerini aynı bellek yuvasında birleştirir.".to_string(),
            "Yüksek eşzamanlıklı Tokio ağ iş yüklerinde çerçeve boyutunu $38\\%$ azaltır ve p99 kuyruk gecikmesini $1.42\\times$ düşürür.".to_string(),
        ));
    }
    if title.contains("Deterministic B-Tree Compaction") {
        return Some((
            "Serileştirilebilir anlık görüntü yalıtımı (SSI) altında deterministik B-ağacı sıkıştırması ile en kötü durum G/Ç karmaşıklığı sınırlandırıldı.".to_string(),
            "Uzun süreli okuma anlık görüntülerinin eski sayfa sürümlerini tuttuğu MVCC depolama motorlarında yazma büyütmesini önlemek için ufuk-bölümlü B-epsilon ağaçları tanıtıldı; NVMe dizilerinde en kötü durum sıkıştırma G/Ç maliyeti $O((B \\log_B N)/M)$ ile sınırlandırıldı.".to_string(),
            "MVCC veritabanlarında uzun süren analitik sorguların eski sayfa sürümlerini kilitleyerek yazma büyütmesine (write amplification) yol açmasını çözer.".to_string(),
            "Anahtar uzayını zaman damgası ufuklarına göre bölümleyen B-epsilon ağaç yapısı ile aktif ve geçmiş sürümleri deterministik olarak ayırır.".to_string(),
            "Sıkı serileştirilebilirlikten ödün vermeden NVMe SSD dizilerinde kuyruk yazma gecikmesini ve G/Ç büyütmesini $O((B \\log_B N)/M)$ sınırında tutar.".to_string(),
        ));
    }
    if title.contains("Byzantine Fault Injection and Split-Brain Recovery") {
        return Some((
            "Mutabakat tabanlı nesne depolarında Bizans hata enjeksiyonu ve bölünmüş-beyin (split-brain) kurtarma analizi.".to_string(),
            "Beş farklı Raft ve Paxos depolama motoru üzerinde yürütülen deterministik simülasyon testleri, asimetrik ağ bölünmeleri ve NVMe `fsync` duraklamaları altında bayat okuma (stale read) hatalarını ortaya çıkardı; geliştirilen TLA+ değişmez denetleyicisi doğrusallaştırılabilirlik ihlallerini $40\\text{ saniye}$ altında yeniden üretir.".to_string(),
            "Dağıtık mutabakat motorlarında asimetrik ağ bölünmeleri ile disk G/Ç gecikmelerinin üst üste bindiği durumlarda oluşan doğrusallaştırılabilirlik ihlallerini inceler.".to_string(),
            "Deterministik simülasyon testini biçimsel TLA+ durum uzayı değişmez (invariant) denetimiyle birleştirerek hatalı lider değişim izlerini yakalar.".to_string(),
            "Üretim sınıfı Raft/Paxos motorlarındaki bölünmüş-beyin kurtarma hatalarını $40\\text{ saniyelik}$ simülasyon süresinde deterministik olarak tespit eder.".to_string(),
        ));
    }

    // II. AI
    if title.contains("Sub-Quadratic Sparse Attention") {
        return Some((
            "1 milyon token bağlamda Blok-Spektral Yönlendirme (BSR) ile $O(L \\log L)$ karmaşıklığında alt-karesel seyrek dikkat mekanizması.".to_string(),
            "Standart öz-dikkat mekanizmasının $O(L^2)$ darboğazını aşmak için anahtar-değer bloklarını düşük dereceli Chebyshev tabanına izdüşüren Blok-Spektral Yönlendirme (BSR), FlashAttention-3 doğruluğunu korurken KV-önbellek bant genişliğini $4.7\\times$ azaltır.".to_string(),
            "Milyon tokenlık uzun bağlamlı çıkarımda tam öz-dikkat mekanizmasının $O(L^2)$ bellek ve hesaplama darboğazını hedef alır.".to_string(),
            "Anahtar-değer bloklarını Chebyshev polinom tabanında spektral kovalar halinde özetleyip sorguları yalnızca en yüksek enerjili $k$ bloğa $O(L \\log L)$ zamanda yönlendirir.".to_string(),
            "Kod muhakemesi ve uzun bağlam testlerinde tam dikkat doğruluğunu korurken KV-önbellek bellek trafiğini $4.7\\times$ düşürür.".to_string(),
        ));
    }
    if title.contains("Verified Tool-Use Trajectories") {
        return Some((
            "Çok adımlı kodlama ajanları için biçimsel önkoşullarla doğrulanmış araç kullanım yörüngeleri.".to_string(),
            "Otonom kodlama ajanlarının hatalı ara hipotezlerde yıkıcı kabuk veya dosya sistemi işlemleri yürütmesini engellemek için MCP araç şemalarına Hoare tarzı durum önkoşulları eklendi; geçersiz eylem dalları yürütme öncesinde budanarak SWE-bench Verified başarısı $32\\%$ daha az token ile $11.4\\%$ artırıldı.".to_string(),
            "Çok adımlı otonom yazılım ajanlarının yanlış hipotez dallarında dosya sistemini bozması ve gereksiz token harcaması sorununu çözer.".to_string(),
            "Araç çağrısı şemalarına hafif sıklet Hoare mantığı önkoşulları ekleyerek durum geçişlerini yürütme öncesinde statik olarak doğrular.".to_string(),
            "SWE-bench Verified üzerinde çözüm oranını $11.4\\%$ yükseltirken toplam token ve araç çağrısı maliyetini $32\\%$ düşürür.".to_string(),
        ));
    }
    if title.contains("Sharp Non-Asymptotic Generalization Bounds for LoRA") {
        return Some((
            "Ağır kuyruklu gradyanlar altında LoRA ince ayarı için keskin asimptotik-olmayan PAC-Bayes genelleme sınırları.".to_string(),
            "Düşük dereceli uyarlamanın (LoRA) küçük alan derlemlerinde ve yüksek gradyan varyansında aşırı uyuma (overfitting) neden direndiği, kararlı derece $\\text{sr}(BA) \\ll r$ ile ölçeklenen asimptotik-olmayan PAC-Bayes sınırlarıyla kanıtlandı.".to_string(),
            "LoRA adaptörlerinin küçük veri kümelerinde tam model ince ayarına kıyasla neden daha yüksek genelleme kararlılığı gösterdiğini matematiksel olarak açıklar.".to_string(),
            "Ağır kuyruklu stokastik gradyan gürültüsü altında adaptör matrislerinin kararlı derecesine ($\text{sr}(BA)$) dayalı PAC-Bayes sınırları türetir.".to_string(),
            "Derece-$16$ ($r=16$) adaptörlerin örneklem karmaşıklığında tam ince ayarı yakaladığını ve ezberlemeyi bastırdığını doğrular.".to_string(),
        ));
    }

    // III. Math
    if title.contains("Improved Polymath Bounds") {
        return Some((
            "$\\mathbb{F}_p^n$ içinde aritmetik dizi içermeyen yoğun kümeler için iyileştirilmiş $(cp)^n$ üst sınırı ve Lean 4 ile biçimsel doğrulama.".to_string(),
            "Dilim-derecesi (slice-rank) polinom spektral kararlılığı kullanılarak $\\mathbb{F}_p^n$ içindeki üç terimli aritmetik dizi içermeyen altkümeler için $c < 0.841$ ile $(cp)^n$ üst sınırı kanıtlandı; temel kombinatorik indirgeme Lean 4 / Mathlib üzerinde bütünüyle mekanize edildi.".to_string(),
            "Sonlu cisimler üzerindeki vektör uzaylarında üç terimli aritmetik dizi içermeyen (cap-set) kümelerin maksimum yoğunluk sınırını sıkılaştırır.".to_string(),
            "Dilim-derecesi polinom yöntemi ile spektral kararlılık lemmasını birleştirir ve tüm kanıt ağacını Lean 4 / Mathlib üzerinde doğrular.".to_string(),
            "Klasik Ellenberg-Gijswijt sınırındaki sabiti düşük karakteristikli cisimler için $c < 0.841$ değerine indirir.".to_string(),
        ));
    }
    if title.contains("Global Regularity for Fractional Navier-Stokes") {
        return Some((
            "Lions üssünün altında kesirli Navier-Stokes denklemleri için küresel $C^\\infty$ düzenlilik kanıtı.".to_string(),
            "3 boyutlu sıkıştırılamaz Navier-Stokes denklemlerinde kesirli sönümleme $(-\\Delta)^\\alpha u$ teriminin $\\alpha = 5/4$ sınırının altında tekillik üretip üretmediği sorunu, uzay-zaman girdap filamentleri boyunca nicel De Giorgi-Nash-Moser salınım sönümü kullanılarak $\\alpha > 5/4 - \\varepsilon_0$ bölgesi için küresel düzenlilikle çözüldü.".to_string(),
            "3B kesirli Navier-Stokes denklemlerinde kritik Lions üssü $\\alpha = 5/4$ altında pürüzsüz çözümlerin patlama (blow-up) yapıp yapmadığı açık problemini ele alır.".to_string(),
            "Girdap (vorticity) filamentleri boyunca nicel De Giorgi-Nash-Moser eşitsizlikleri ve çok ölçekli enerji sönümü uygular.".to_string(),
            "$\\alpha > 5/4 - \\varepsilon_0$ rejiminde sonlu zamanda tekillik oluşmadığını ve küresel $C^\\infty$ düzenliliğin korunduğunu kanıtlar.".to_string(),
        ));
    }
    if title.contains("Effective Chabauty-Kim Method") {
        return Some((
            "Sayı cisimleri üzerinde cinsiyeti $g \\ge 3$ olan hipereliptik eğriler için efektif Chabauty-Kim algoritması.".to_string(),
            "Mordell-Weil derecesinin eğri cinsiyetine ($g$) eşit olduğu durumlarda rasyonel noktaların belirlenmesi için değişmeli-olmayan $p$-adik tekrarlı integraller üzerinden Selmer çeşitlilikleri üzerinde Coleman-Gross yüksekliklerini hesaplayan açık bir algoritma sunuldu ve cinsiyeti $3$ olan hipereliptik eğri ailesinin tüm rasyonel noktaları sertifikalandırıldı.".to_string(),
            "Klasik Chabauty yönteminin uygulanamadığı $r = g \\ge 2$ rejimindeki cebirsel eğriler üzerinde rasyonel noktaların sonlu kümesini bulma problemini çözer.".to_string(),
            "Selmer çeşitlilikleri üzerinde $p$-adik Coleman-Gross yükseklikleri ve değişmeli-olmayan temel grup bölümlerini hesaplar.".to_string(),
            "Cinsiyeti $3$ olan hipereliptik eğri aileleri için rasyonel noktaların tam listesini bilgisayar destekli sertifika ile üretir.".to_string(),
        ));
    }

    // IV. Physics
    if title.contains("Fault-Tolerant Logical Magic-State") {
        return Some((
            "İki değişkenli bisiklet (bivariate bicycle) kuantum LDPC kodları üzerinde sabit derinlikli ve hata-toleranslı sihirli durum damıtımı.".to_string(),
            "Yüzey kodlarının mantıksal kübit başına $1{,}000+$ fiziksel kübit gereksinimini aşmak için $[[144, 12, 12]]$ qLDPC kodları üzerinde doğrudan çalışan sabit derinlikli sihirli durum fabrikası kuruldu; düzlemsel örgü cerrahisine kıyasla $6.8\\times$ daha az fiziksel kübitle $10^{-9}$ altı mantıksal $T$-kapısı hatasına ulaşıldı.".to_string(),
            "Evrensel hata-toleranslı kuantum hesaplamada en büyük fiziksel kübit maliyetini oluşturan sihirli durum (magic-state) damıtım darboğazını çözer.".to_string(),
            "Yüksek oranlı $[[144, 12, 12]]$ iki değişkenli bisiklet qLDPC kodları üzerinde kod-içi (in-place) sabit derinlikli mantıksal ölçüm devreleri tanımlar.".to_string(),
            "Gerçekçi $p \\approx 10^{-3}$ kapı hata oranlarında yüzey kodlarına kıyasla $6.8\\times$ daha az fiziksel kübit kullanarak $10^{-9}$ altı mantıksal hata oranı sağlar.".to_string(),
        ));
    }
    if title.contains("Fractional Quantum Anomalous Hall Superconductivity") {
        return Some((
            "Rombohedral beş katmanlı grafende sıfır manyetik alanda kesirli kuantum anormal Hall süperiletkenliği gözlemlendi.".to_string(),
            "hBN hizalı rombohedral beş katmanlı grafende yapılan taşınım spektroskopisi, sıfır dış manyetik alanda $\\nu = -2/3$ kesirli topolojik Chern yalıtkanı durumundan doğrudan doğan ve $T_c = 380\\text{ mK}$ altında kayıpsız akım taşıyan bir süperiletken kubbe ortaya çıkardı.".to_string(),
            "Sıfır manyetik alanda kesirli Chern yalıtkanları ile alışılmadık süperiletkenliğin aynı faz diyagramında bir arada bulunup bulunamayacağı sorusunu yanıtlar.".to_string(),
            "hBN ile hizalanmış beş katmanlı rombohedral grafen heteroyapılarında milikelvin sıcaklıkta elektrostatik kapılama ve taşınım spektroskopisi uygular.".to_string(),
            "$\\nu = -2/3$ kesirli topolojik durumun hemen komşuluğunda $T_c = 380\\text{ mK}$ kritik sıcaklığa sahip kayıpsız süperiletken faz geçişini doğrular.".to_string(),
        ));
    }
    if title.contains("Conformal Bootstrap Bounds") {
        return Some((
            "$\\text{AdS}_5 \\times S^5$ uzayında graviton saçılma genlikleri üzerinde konformal önyükleme (bootstrap) sınırları.".to_string(),
            "Sonlu 't Hooft kuplajı $\\lambda$ değerlerinde süperkütleçekim genliklerine gelen pertürbatif-olmayan sicim düzeltmeleri, dağılım bağıntıları ve çaprazlama simetrisiyle sınırlandırıldı; sayısal konformal önyükleme ve lokalizasyon girdileri birleştirilerek ilk üç yüksek-türevli Wilson katsayısı tüm kuplajlarda $0.04\\%$ hassasiyetle belirlendi.".to_string(),
            "Sonlu kuplaj rejiminde kuantum kütleçekim saçılma genliklerinin pertürbatif-olmayan sicim düzeltmelerini yüksek hassasiyetle hesaplama problemini ele alır.".to_string(),
            "Süpersimetrik lokalizasyon kısıtlarını hiperbolik dağılım bağıntıları ve sayısal konformal önyükleme (bootstrap) optimizasyonuyla birleştirir.".to_string(),
            "İlk üç yüksek-türevli Wilson katsayısını tüm 't Hooft kuplaj aralığında $0.04\\%$ hata payı içinde sıkıştırır.".to_string(),
        ));
    }

    // V. Systems
    if title.contains("Hardware-Enforced Capability Tagging") {
        return Some((
            "Çekirdek ağ halkalarında (io_uring / DMA) donanım destekli yetenek etiketlemesi ile Use-After-Free açıklıklarının deterministik olarak engellenmesi.".to_string(),
            "Linux çekirdeğindeki yüksek hızlı `io_uring` ve sıfır-kopya DMA halkalarını IOMMU sınırında doğrulanan CHERI/MTE bellek yetenek etiketlerine bağlayan mimari, $400\\text{ Gb/s}$ hat hızında yalnızca $1.8\\%$ ek yükle zamansal bellek takma adı (temporal aliasing) hatalarını yakalar.".to_string(),
            "Linux çekirdeğinde asenkron G/Ç (`io_uring`) ve paylaşımlı DMA halkalarında sıkça istismar edilen sayfa düzeyi Use-After-Free güvenlik açıklarını hedef alır.".to_string(),
            "Tanımlayıcı halkalarını işlemci ve IOMMU tarafından donanımsal olarak denetlenen ince taneli CHERI/MTE bellek yetenek (capability) etiketleriyle eşleştirir.".to_string(),
            "$400\\text{ Gb/s}$ ağ iş yüklerinde paket işleme hacmini temel çizginin $1.8\\%$ dahilinde tutarken zamansal bellek ihlallerini deterministik olarak durdurur.".to_string(),
        ));
    }
    if title.contains("Open-Source Out-of-Order RISC-V Vector Core") {
        return Some((
            "Spekülatif matris yazmaç yeniden adlandırmalı açık kaynaklı sırasız (out-of-order) RISC-V vektör çekirdeği.".to_string(),
            "Karma FP8/BF16 çekirdeklerinde yazmaç dosyası portlarının doygunluğa ulaşarak süperskaler boru hatlarını durdurmasını önlemek için matris akümülatör yeniden adlandırmasını skaler ROB girdilerinden ayıran açık kaynaklı SystemVerilog `RV64GCV` çekirdeği $5\\text{ nm}$ sürecinde üretildi ve $3.1\\text{ TFLOPS/W}$ verime ulaştı.".to_string(),
            "Uzun vektör ve matris komutlarının süperskaler işlemcilerde yazmaç dosyası port darboğazı nedeniyle boru hattı duraklamalarına yol açmasını çözer.".to_string(),
            "Matris akümülatör yeniden adlandırma tablosunu skaler yeniden sıralama tamponundan (ROB) ayıran çift katmanlı `RV64GCV` mikro-mimarisi sunar.".to_string(),
            "$5\\text{ nm}$ üretim sürecinde $3.1\\text{ TFLOPS/W}$ enerji verimliliği ve açık kaynaklı RTL doğrulama paketi sağlar.".to_string(),
        ));
    }
    if title.contains("Cross-Cache Speculative Type Confusion") {
        return Some((
            "Modern tarayıcı JIT derleyici katmanlarında önbellekler-arası spekülatif tip karışıklığı (type confusion) ve sıfır-maliyetli bariyer çözümü.".to_string(),
            "Arka planda eşzamanlı JIT derlemesi sırasında monomorfik nesne şekil varsayımlarının spekülatif yarış koşuluna girmesiyle site yalıtım sınırları arasında $90\\text{ ms}$ altında keyfi WebAssembly doğrusal belleği sızdırabildiği gösterildi ve sıfır ek yüklü bellek bariyeri yaması sunuldu.".to_string(),
            "Tarayıcı JIT derleyicilerinde profil geri bildirimine dayalı sınır/tip denetimi elemelerinin spekülatif yürütme altında yarattığı bellek sızıntılarını inceler.".to_string(),
            "Arka plan derleyici iş parçacığı ile ana JS yürütme hattı arasındaki harita geçiş yarışını (race condition) spekülatif önbellek yan kanalıyla birleştirir.".to_string(),
            "Sızıntıyı $90\\text{ ms}$ altında kanıtlar ve çalışma zamanı performansı düşürmeyen derleyici bariyer (fence) çözümünü doğrular.".to_string(),
        ));
    }

    // VI. Bio
    if title.contains("All-Atom Equivariant Flow Matching") {
        return Some((
            "De novo metaloenzim aktif bölge tasarımı için tüm-atom $SE(3)$-eşdeğişkin akış eşleştirme (flow matching) ve kuantum mekaniksel potansiyel kuplajı.".to_string(),
            "Geçiş metali koordinasyon geometrisinde $3d$-orbital elektronik etkilerini modellemek için $SE(3)$-eşdeğişkin akış eşleştirmeyi sıkı-bağlama (tight-binding) kuantum mekaniksel potansiyelle birleştiren model, yönlendirilmiş evrim olmadan ıslak laboratuvar deneylerinde $k_{\\text{cat}}/K_M > 10^5\\text{ M}^{-1}\\text{s}^{-1}$ katalitik verime sahip çinko ve demir hidrolazlar tasarlar.".to_string(),
            "Standart protein difüzyon modellerinin geçiş metali kofaktörlerinin kuantum mekaniksel koordinasyon geometrisini doğru üretememesi sorununu çözer.".to_string(),
            "Tüm-atom $SE(3)$-eşdeğişkin akış eşleştirme mimarisini aktif bölge çevresinde yarı-ampirik sıkı-bağlama kuantum potansiyeli ile ortaklaşa optimize eder.".to_string(),
            "Deneysel olarak sentezlenen de novo metaloenzimlerde yönlendirilmiş evrime gerek kalmadan $k_{\\text{cat}}/K_M > 10^5\\text{ M}^{-1}\\text{s}^{-1}$ aktivite elde edilir.".to_string(),
        ));
    }
    if title.contains("Pangenome Graph Indexing in Sub-Linear Space") {
        return Some((
            "Wheeler dalgacık ağaçları ile alt-doğrusal bellekte insan pangenom çizgesi dizinleme ve hızlı Nanopore hizalama.".to_string(),
            "Binlerce haplotip içeren popülasyon ölçekli insan pangenom çizgelerinin $100\\text{ GB}$ üstü RAM gereksinimini aşmak için çift yönlü FM-dizinleri üzerinde sıkıştırılmış Wheeler çizge dizini kuruldu; uzun Nanopore okumaları yalnızca $4.2\\text{ GB}$ bellekte Minigraph-Cactus'tan $3.4\\times$ daha hızlı hizalanır.".to_string(),
            "Popülasyon ölçekli pangenom referans çizgelerinde geleneksel $k$-mer tohum tablolarının aşırı bellek tüketimini ortadan kaldırır.".to_string(),
            "Çift yönlü FM-dizinleri üzerinde Wheeler bürokrat dalgacık ağaçları (wavelet trees) kullanarak kayıpsız sıkıştırılmış alt-doğrusal arama yapısı kurar.".to_string(),
            "Bellek ayak izini $100\\text{ GB}$ seviyesinden $4.2\\text{ GB}$ düzeyine indirirken uzun okuma hizalama hızını $3.4\\times$ artırır.".to_string(),
        ));
    }
    if title.contains("Neural Phonon Boltzmann Transport Solver") {
        return Some((
            "Katı hal lityum elektrolitlerinde ultra-düşük ısıl iletkenlik fazlarını bulan eşdeğişkin nöral fonon Boltzmann çözücüsü.".to_string(),
            "İlk-ilkeler anharmonik örgü dinamiğinin $O(N_{\\text{atoms}}^4)$ hesaplama maliyetini aşan eşdeğişkin fonon nöral operatörü, tam ısıl iletkenlik tensörlerini DFT-BTE'ye göre $4\\%$ hata payı içinde ve $10^4\\times$ daha düşük maliyetle tahmin ederek üç yeni kararlı arjirodit katı elektrolit fazı keşfetti.".to_string(),
            "Katı hal batarya malzemelerinde 3- ve 4-fonon saçılma hesaplarının yüksek atom sayılarında $O(N^4)$ ölçeklenmesi darboğazını çözer.".to_string(),
            "Kristal simetri grubuna saygılı eşdeğişkin sinirsel operatör ile doğrusallaştırılmış fonon Boltzmann taşınım denklemini (BTE) doğrudan çözer.".to_string(),
            "DFT-BTE doğruluğunu $10^4\\times$ daha hızlı yakalayarak yüksek iyonik iletkenlikli üç yeni arjirodit fazını belirler.".to_string(),
        ));
    }

    // VII. Crypto
    if title.contains("HyperFold: Prover-Optimal Lattice Folding") {
        return Some((
            "HyperFold: Goldilocks asal halkaları üzerinde kuantum-sonrası güvenli ve $O(\\log N)$ doğrulamalı kafes katlama (lattice folding) protokolü.".to_string(),
            "Shor algoritmasına duyarlı eliptik eğri katlama şemalarının (Nova/HyperNova) yerine Goldilocks asalları ($p = 2^{64} - 2^{32} + 1$) üzerinde kurulan ring-SIS kafes katlama protokolü, $128$-bit kuantum-sonrası güvenlik, $14\\text{ KB}$ kanıt boyutu ve tüketici GPU'larında $3.9\\times$ daha hızlı RISC-V zkVM kanıtlama performansı sunar.".to_string(),
            "Mevcut eliptik eğri tabanlı IVC/katlama şemalarının kuantum saldırılarına açıklığını ve SHA-3/Keccak devrelerindeki ağır yerel-olmayan cisim aritmetiğini ortadan kaldırır.".to_string(),
            "$p = 2^{64} - 2^{32} + 1$ Goldilocks halkası üzerinde norm kontrollü ring-SIS taahhüt katlama şeması ve doğrulayıcı-optimal toplam kontrolü (sumcheck) inşa eder.".to_string(),
            "$128$-bit kuantum-sonrası güvenlik altında kanıt boyutunu $14\\text{ KB}$ seviyesinde tutar ve RISC-V zkVM kanıtlama hızını $3.9\\times$ artırır.".to_string(),
        ));
    }
    if title.contains("Single-Slot Finality under Sub-Sampling DAS") {
        return Some((
            "Eşik BLS imza toplama ağaçları ve PeerDAS veri örneklemesi altında tek-yuva kesinliği (Single-Slot Finality).".to_string(),
            "$10^6$ üstü doğrulayıcıya sahip BFT mutabakatında $12$ saniyelik tek yuvada kesinlik sağlamak için silme kodlamalı veri erişilebilirliği örneklemesi (PeerDAS) ile oransız cebirsel BLS imza birleştirme ağaçları birleştirildi; düğüm başına bant genişliği $4.5\\text{ MB/s}$ altında tutularak iki turlu kesinlik garanti edildi.".to_string(),
            "Milyon ölçekli doğrulayıcı kümelerinde tek yuvada (12 saniye) BFT mutabakat kesinliğini engelleyen imza trafiği ve dedikodu (gossip) darboğazını çözer.".to_string(),
            "PeerDAS veri erişilebilirliği alt-örneklemesini cebirsel sıçramalı (rateless) BLS eşik imza toplama ağaçlarıyla birleştirir.".to_string(),
            "Düğüm başına ağ trafiğini $4.5\\text{ MB/s}$ sınırının altında tutarak $12$ saniyelik tek yuvada iki turlu kesinliği kanıtlar.".to_string(),
        ));
    }
    if title.contains("Symbolic & SMT-Guided Equivalence Checking") {
        return Some((
            "EVM bayt kodu ile ZK-devre derleyicileri arasında sembolik ve SMT destekli bit-kesin eşdeğerlik doğrulaması.".to_string(),
            "zkSNARK devrelerindeki eksik kısıtlanmış aritmetik kapıları ve Solidity/Yul derleyici hatalarını yakalamak için geliştirilen SMT destekli ilişkisel doğrulayıcı, optimize edilmiş EVM bayt kodu ile Plonkish kısıt sistemleri arasındaki bit-kesin eşdeğerliği döngü-sınırlı sözleşmeler için polinom zamanda kanıtlar.".to_string(),
            "Sıfır-bilgi (ZK) derleyicilerindeki eksik kısıtlamalar ve EVM optimizasyon geçişlerindeki depolama örtüşmesi kaynaklı kritik protokol açıklarını hedef alır.".to_string(),
            "EVM yürütme semantiği ile Plonkish aritmetik devre kısıtlarını ortak bir ilişkisel Hoare mantığı ve SMT çözücüsüne indirger.".to_string(),
            "Döngü-sınırlı akıllı sözleşmelerde bayt kodu ile ZK devresi arasındaki bit-kesin eşdeğerliği otomatik olarak sertifikalandırır.".to_string(),
        ));
    }

    // VIII. Economics, Markets & Quantitative Finance
    if title.contains("Heterogeneous Agent Monetary Transmission") {
        return Some((
            "Kamu borcu vade duvarları altında heterojen ajanlı (HANK) parasal aktarım mekanizması ve mali baskınlık sınırları.".to_string(),
            "38 OECD ve gelişmekte olan ülke tahvil piyasası üzerinde içsel vade primli sürekli-zamanlı bir HANK modeli çözüldü; kısa vadeli borç çevirme oranının GSYH'nin $28\\%$'ini aştığı ekonomilerde $100\\text{ baz puanlık}$ faiz artışının mali baskınlık yayılımları nedeniyle dezenflasyon etkisini $42\\%$ zayıflattığı, ancak likidite çekici repo koridorlarıyla bu kaybın telafi edildiği gösterildi.".to_string(),
            "Yüksek kısa vadeli kamu borcu çevirme yükü altında merkez bankası faiz artışlarının vade primi ve mali bütçe kanalıyla dezenflasyonu nasıl zayıflattığını inceler.".to_string(),
            "İçsel getiri eğrisi primleri ve doğrusal-olmayan mali tepki fonksiyonları içeren sürekli-zamanlı heterojen ajanlı Yeni Keynesyen (HANK) modeli çözer.".to_string(),
            "Kısa vadeli borç yenileme duvarının GSYH'nin $28\\%$'ini geçtiği noktada parasal aktarımın $42\\%$ aşındığını ve sterilize repo koridorlarının gerekliliğini ortaya koyar.".to_string(),
        ));
    }
    if title.contains("Optimal Execution and Adverse Selection in Hawkes-Driven") {
        return Some((
            "Pürüzlü oynaklık (rough volatility) ve Hawkes süreçli limit emir defterlerinde optimal emir yürütme ve ters seçim.".to_string(),
            "Klasik Almgren-Chriss modelinin likidite kuruması anlarında kuyruk kaymasını (slippage) eksik tahmin etmesini aşmak için emir akışı çift değişkenli öz-uyarımlı Hawkes süreci ve kesirli Hurst üssü $H \\approx 0.12$ ile modellendi; kapalı-form Riccati yürütme çizelgeleri uygulama açığını (implementation shortfall) $19.4\\text{ baz puan}$ düşürür.".to_string(),
            "Elektronik piyasalarda yüksek frekanslı emir kümelenmesi ve pürüzlü oynaklık sırasında büyük kurumsal emirlerin maruz kaldığı ters seçim maliyetini minimize eder.".to_string(),
            "Karşılıklı uyarımlı iki değişkenli Hawkes nokta süreçlerini $H \\approx 0.12$ kesirli oynaklık çekirdeğiyle birleştirerek Riccati kontrol denklemlerini çözer.".to_string(),
            "Geleneksel TWAP/Almgren-Chriss çizelgelerine kıyasla gerçekleşen uygulama maliyetini ortalama $19.4\\text{ bps}$ azaltır.".to_string(),
        ));
    }
    if title.contains("Supply-Chain Network Topology and Pass-Through") {
        return Some((
            "Tedarik zinciri ağ topolojisi ve dezenflasyon dönemlerinde girdi-çıktı fiyat geçişkenliği asimetrileri.".to_string(),
            "$1.4\\times 10^6$ tedarikçi bağlantısını kapsayan firmalar-arası KDV fatura çizgeleri üzerinde yapılan parametrik-olmayan tahminde, yüksek Katz merkeziliğine sahip yukarı-akım oligopolistik darboğazların parasal sıkılaşma sonrasında tüketici fiyat istikrarını rekabetçi sektörlere kıyasla $2.6$ çeyrek geciktirdiği kanıtlandı.".to_string(),
            "Parasal sıkılaşma döngülerinde üretim ağı topolojisinin ve tedarikçi yoğunlaşmasının dezenflasyon hızını neden geciktirdiğini mikro verilerle analiz eder.".to_string(),
            "$1.4$ milyon firma-firma fatura bağlantısı üzerinde Katz ağ merkeziliği ve parametrik-olmayan girdi-çıktı fiyat katılığı tahmincisi uygular.".to_string(),
            "Merkezi tedarik düğümlerindeki marj katılığının aşağı-akım TÜFE dezenflasyonunu ortalama $2.6$ çeyrek geciktirdiğini ortaya koyar.".to_string(),
        ));
    }

    // IX. International Relations, Geopolitics & Strategy
    if title.contains("Chokepoint Geometries in Sub-2nm EUV Lithography") {
        return Some((
            "2nm altı EUV litografi ve yüksek bant genişlikli bellek (HBM) tedarik zincirlerinde darboğaz geometrisi.".to_string(),
            "Yalnızca nihai yapay zekâ hızlandırıcılarının FLOPS/mm² yoğunluğunu hedef alan ihracat kontrollerinin, yukarı-akım fotorezist kimyasalları, EUV pelikülleri ve gelişmiş paketleme (CoWoS-L) tek-nokta bağımlılıklarını gözden kaçırdığı; 140 üretim katmanlı ağ modelinde plaka-düzeyi metroloji kısıtlarının sınır küme kurulumunu fabrika ekipmanı ambargolarından $3.2\\times$ daha sıkı sınırladığı gösterildi.".to_string(),
            "Yarı iletken ihracat kontrollerinde hesaplama gücü eşiklerinin tedarik zincirindeki kimyasal ve gelişmiş paketleme darboğazlarıyla nasıl etkileştiğini inceler.".to_string(),
            "140 yarı iletken üretim, litografi kimyasalı, EUV pelikül ve CoWoS-L paketleme katmanı üzerinde girdi-çıktı darboğaz çizgesi kurar.".to_string(),
            "Metroloji ve ileri paketleme darboğazlarının sınır ölçekli YZ veri merkezi kurulumlarını ekipman kısıtlarından $3.2\\times$ daha güçlü biçimde belirlediğini gösterir.".to_string(),
        ));
    }
    if title.contains("Attritional Cost-Exchange Ratios of Autonomous Loitering") {
        return Some((
            "Çok katmanlı hava savunmasına karşı otonom dolanan mühimmat sürülerinin yıpratma ve maliyet-değişim oranları.".to_string(),
            "Önleyici füze ile hedef birim maliyet oranının $25:1$'i aşması doygunluk salvoları altında yapısal mühimmat derinliği tükenmesine yol açmaktadır; nicel salvo modellemesi, kısa menzilli yönlendirilmiş enerji sistemleri ile düşük maliyetli akustik/RF sensör ağlarının entegrasyonunun dakikada $180\\text{ iz}$ yoğunluğuna kadar maliyet-değişim dengesini savunma lehine çevirdiğini göstermektedir.".to_string(),
            "Düşük maliyetli otonom İHA/mühimmat sürülerinin geleneksel karadan-havaya füze (SAM) bataryalarını ekonomik ve lojistik olarak tüketmesi sorununu analiz eder.".to_string(),
            "Stokastik salvo-kesişme simülasyonu ile kinetik önleyiciler, yönlendirilmiş enerji silahları ve dağıtık pasif RF/akustik sensör ağlarını birlikte modeller.".to_string(),
            "Katmanlı hibrit savunma mimarisinin dakikada $180$ hedefe kadar birim önleme maliyetini tersine çevirerek mühimmat tükenmesini önlediğini kanıtlar.".to_string(),
        ));
    }
    if title.contains("Subsea HVDC Interconnectors, Critical Mineral Corridors") {
        return Some((
            "Denizaltı HVDC enterkonnektörleri, kritik mineral koridorları ve Orta Koridor şebeke egemenliği.".to_string(),
            "Karadeniz, Kafkasya ve Doğu Akdeniz üzerinden geçen kıtalararası yüksek gerilim doğru akım (HVDC) kabloları ve nadir toprak elementleri işleme koridorlarının bölgesel caydırıcılığı nasıl yeniden şekillendirdiği incelendi; oyun-kuramsal pazarlık analizi, çok-düğümlü transit merkezlerinin kış zirve yük senaryolarında tek-tedarikçi zorlama kaldıracını $54\\%$ azalttığını kanıtlar.".to_string(),
            "Sınır-aşan elektrik iletim hatları ve kritik mineral tedarik koridorlarının enerji güvenliği ile jeopolitik pazarlık gücüne etkisini inceler.".to_string(),
            "Çok-düğümlü enerji ve lojistik şebekeleri üzerinde kış tepe yükü stres testleri ve Nash-Shapley pazarlık gücü endeksleri hesaplar.".to_string(),
            "Orta Koridor ve denizaltı HVDC bağlantılarının tekil tedarikçi kesinti şantajını $54\\%$ oranında zayıflattığını ortaya koyar.".to_string(),
        ));
    }

    // X. Philosophy of Science, Decision Theory & Cognition
    if title.contains("Imprecise Probabilities, Dutch-Book Coherence") {
        return Some((
            "Ciddi model yanlış-belirtimi altında kesin-olmayan olasılıklar (imprecise probabilities), Dutch-Book tutarlılığı ve epistemik tevazu.".to_string(),
            "Klasik Bayesçi koşullandırma, ajanın önsel dağılım desteği gerçek veri-üretim sürecini dışladığında yanlış sınır hipotezlerine aşırı özgüvenle yakınsar; $\\alpha$-ıraksama güncellemeleri altında dışbükey inanç kümeleri (credal sets) için kanıtlanan temsil teoremi, dinamik tutarlılığı korurken yapısal model hatasının asimptotik tespitini garanti eder.".to_string(),
            "Standart Bayesçi öğrenmenin yanlış tanımlanmış (misspecified) modellerde yanlış hipotezlere aşırı kesinlikle kilitlenmesi problemini ele alır.".to_string(),
            "Dışbükey olasılık kümeleri (credal sets) üzerinde $\\alpha$-ıraksama tabanlı genelleştirilmiş güncelleme kuralları ve Dutch-Book tutarlılık teoremi kurar.".to_string(),
            "Dinamik tutarlılıktan ödün vermeden yapısal model hatasını asimptotik olarak yakalayan epistemik tevazu sınırlarını kanıtlar.".to_string(),
        ));
    }
    if title.contains("Coarse-Graining and Causal Emergence") {
        return Some((
            "Çok ölçekli Markov battaniyelerinde kaba-ölçekleme (coarse-graining) ve nedensel beliriş (causal emergence).".to_string(),
            "Bir makro-durum değişkeninin altta yatan mikro-dinamiklerden ne zaman daha yüksek nedensel öngörü gücüne sahip olduğu sorusu, hiyerarşik Markov battaniyeleri üzerinde müdahaleci bilgi kuramı ve renormalizasyon grubu akışlarıyla çözüldü; dejenere mikro-gürültü sönümlenmesi sayesinde makro-ölçek efektif bilginin $EI(M) > EI(\\mu)$ olduğu tam eşik türetildi.".to_string(),
            "Karmaşık biyolojik ve fiziksel sistemlerde üst-düzey makro açıklamaların mikro-durumlara kıyasla ne zaman indirgenemez nedensel güç kazandığını biçimselleştirir.".to_string(),
            "Hiyerarşik Markov battaniyeleri üzerinde Judea Pearl'ün müdahale (do-calculus) operatörlerini renormalizasyon grubu kaba-ölçeklemesiyle birleştirir.".to_string(),
            "Makro-ölçek efektif bilginin ($EI(M)$) mikro-ölçeği ($EI(\\mu)$) aştığı analitik faz sınırını türetir.".to_string(),
        ));
    }
    if title.contains("Compositional Generalization Limits in Connectionist") {
        return Some((
            "Bağlantıcı (connectionist) mimarilerde bileşimsel genelleme sınırları ve sembolik program tümevarımı karşılaştırması.".to_string(),
            "Bilişsel sistematiklik ve üretkenlik için gereken cebirsel rol-değer bağımsızlığının, standart otoregresif dizi modellerinde yalnızca eğitim dışbükey zarfı içinde yaklaşıklandığı; minimum açıklama uzunluğu (MDL) tabanlı Bayesçi program öğreniciler ile dönüştürücü (transformer) durum-izleme devreleri kıyaslandığında $d \\ge 7$ özyineleme derinliğinde keskin bir faz geçişi yaşandığı gösterildi.".to_string(),
            "Sinir ağlarının eğitim dağılımı dışındaki (OOD) özyinelemeli ve bileşimsel kuralları neden belirli bir derinlikten sonra genelleyemediğini inceler.".to_string(),
            "MDL tabanlı sembolik Bayesçi program tümevarımını dönüştürücü (transformer) dikkat devrelerinin durum izleme kapasitesiyle biçimsel olarak karşılaştırır.".to_string(),
            "Dağılım-dışı özyineleme derinliği $d \\ge 7$ olduğunda otoregresif devrelerde keskin bir doğruluk çöküşü (faz geçişi) gerçekleştiğini kanıtlar.".to_string(),
        ));
    }

    None
}

fn split_sentences(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        current.push(chars[i]);
        if matches!(chars[i], '.' | '!' | '?') {
            let next_is_space_or_end = i + 1 == chars.len() || chars[i + 1].is_whitespace();
            if next_is_space_or_end && current.trim().len() > 28 {
                out.push(current.trim().to_string());
                current.clear();
            }
        }
        i += 1;
    }
    if !current.trim().is_empty() {
        out.push(current.trim().to_string());
    }
    out
}

fn truncate_at_word(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }
    let slice = truncate_utf8_safe(s, max_len);
    if let Some(pos) = slice.rfind(' ') {
        format!("{}…", &slice[..pos])
    } else {
        format!("{}…", slice)
    }
}

fn truncate_utf8_safe(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

fn normalize_url(u: &str) -> String {
    u.trim()
        .to_ascii_lowercase()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.")
        .trim_end_matches('/')
        .to_string()
}

fn make_item_id(issue_date: &str, seq: usize, title: &str) -> String {
    let slug: String = title
        .to_ascii_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .take(5)
        .collect::<Vec<_>>()
        .join("-");
    format!("{}-{:02}-{}", issue_date, seq, slug)
}


#[derive(Debug, Default, Clone)]
pub struct AcademicGraphData {
    pub citation_count: Option<u32>,
    pub influential_citations: Option<u32>,
    pub author_institutions: Option<Vec<String>>,
}

pub fn extract_arxiv_id(url: &str) -> Option<String> {
    if !url.contains("arxiv.org") {
        return None;
    }
    let parts: Vec<&str> = url.split("/abs/").collect();
    let id_part = if parts.len() > 1 {
        parts[1]
    } else {
        let pdf_parts: Vec<&str> = url.split("/pdf/").collect();
        if pdf_parts.len() > 1 {
            pdf_parts[1]
        } else {
            return None;
        }
    };
    let clean = id_part.trim_end_matches(".pdf").trim();
    if clean.is_empty() {
        return None;
    }
    let no_v = if let Some(v_idx) = clean.rfind('v') {
        if v_idx > 0 && clean[v_idx + 1..].chars().all(|ch| ch.is_ascii_digit()) {
            &clean[..v_idx]
        } else {
            clean
        }
    } else {
        clean
    };
    Some(no_v.to_string())
}

pub async fn fetch_academic_graph_metadata(
    client: &Client,
    arxiv_id: &str,
) -> Option<AcademicGraphData> {
    let url = format!(
        "https://api.semanticscholar.org/graph/v1/paper/arXiv:{}?fields=citationCount,influentialCitationCount,authors.affiliations",
        arxiv_id
    );

    let resp = client
        .get(&url)
        .header(
            "User-Agent",
            "Ahbar/0.1 (https://latifkedi.github.io/ahbar; mailto:contact@ahbar.org)",
        )
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let json: serde_json::Value = resp.json().await.ok()?;

    let citation_count = json.get("citationCount").and_then(|v| v.as_u64()).map(|n| n as u32);
    let influential_citations = json.get("influentialCitationCount").and_then(|v| v.as_u64()).map(|n| n as u32);

    let mut inst_set = HashSet::new();
    if let Some(authors) = json.get("authors").and_then(|v| v.as_array()) {
        for a in authors {
            if let Some(affils) = a.get("affiliations").and_then(|v| v.as_array()) {
                for af in affils {
                    if let Some(s) = af.as_str() {
                        let trimmed = s.trim();
                        if !trimmed.is_empty() && trimmed.len() <= 60 {
                            inst_set.insert(trimmed.to_string());
                        }
                    }
                }
            }
        }
    }

    let author_institutions = if inst_set.is_empty() {
        None
    } else {
        let mut list: Vec<String> = inst_set.into_iter().collect();
        list.sort();
        Some(list)
    };

    Some(AcademicGraphData {
        citation_count,
        influential_citations,
        author_institutions,
    })
}

fn generate_bibtex(
    id: &str,
    title: &str,
    authors: &[String],
    url: &str,
    source_name: &str,
    issue_date: &str,
) -> String {
    let year = issue_date.split('-').next().unwrap_or("2026");
    let author_str = if authors.is_empty() {
        source_name.to_string()
    } else {
        authors.join(" and ")
    };
    let cite_key: String = id.replace('-', "_");
    format!(
        "@article{{{},\n  title = {{{}}},\n  author = {{{}}},\n  year = {{{}}},\n  journal = {{{}}},\n  url = {{{}}}\n}}",
        cite_key, title, author_str, year, source_name, url
    )
}
