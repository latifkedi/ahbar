use std::collections::HashSet;

/// Anti-sponsor, anti-ad, and anti-clickbait filter.
/// Rejects promotional posts, affiliate links, webinars, and low-signal fluff
/// before they ever reach the LLM or feed.
pub fn is_sponsored_or_clickbait(title: &str, summary: &str, url: &str) -> bool {
    let t_low = title.to_ascii_lowercase();
    let s_low = summary.to_ascii_lowercase();
    let u_low = url.to_ascii_lowercase();

    // URL-level promotional markers
    let bad_url_markers = [
        "utm_medium=sponsored",
        "utm_campaign=sponsor",
        "utm_source=ad",
        "/sponsored/",
        "/partner-content/",
        "/advertorial/",
        "affiliate=",
    ];
    if bad_url_markers.iter().any(|m| u_low.contains(m)) {
        return true;
    }

    // Title-level promotional or clickbait markers
    let bad_title_markers = [
        "[sponsored]",
        "(sponsored)",
        "sponsored:",
        "partner content:",
        "register for our webinar",
        "join our free webinar",
        "black friday deal",
        "cyber monday",
        "limited time offer",
        "you won't believe",
        "10 tools that will blow your mind",
        "why everyone is switching to",
    ];
    if bad_title_markers.iter().any(|m| t_low.contains(m)) {
        return true;
    }

    // Summary-level ad disclosures
    let bad_summary_markers = [
        "this post is sponsored by",
        "thank you to our sponsor",
        "in partnership with",
        "use coupon code",
    ];
    if bad_summary_markers.iter().any(|m| s_low.contains(m)) {
        return true;
    }

    false
}

/// Validates and repairs inline LaTeX delimiters (`$...$` and `$$...$$`)
/// so unclosed math markers never corrupt paragraph rendering.
pub fn sanitize_and_validate_latex(input: &str) -> String {
    let mut text = input.trim().to_string();
    if text.is_empty() {
        return text;
    }

    // Normalize `\( ... \)` into `$...$` and `\[ ... \]` into `$$...$$`
    text = text.replace("\\(", "$").replace("\\)", "$");
    text = text.replace("\\[", "$$").replace("\\]", "$$");

    // Count non-escaped single `$` signs (ignoring `$$` and `\$`)
    let chars: Vec<char> = text.chars().collect();
    let mut single_dollar_positions = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' {
            i += 2;
            continue;
        }
        if chars[i] == '$' {
            if i + 1 < chars.len() && chars[i + 1] == '$' {
                i += 2;
                continue;
            }
            single_dollar_positions.push(i);
        }
        i += 1;
    }

    // If there is an odd number of unescaped single `$` delimiters, escape the stray last one
    if single_dollar_positions.len() % 2 == 1 {
        if let Some(&last_pos) = single_dollar_positions.last() {
            let mut repaired = String::with_capacity(text.len() + 1);
            for (idx, ch) in chars.iter().enumerate() {
                if idx == last_pos {
                    repaired.push('\\');
                }
                repaired.push(*ch);
            }
            text = repaired;
        }
    }

    text
}

/// Computes a 64-bit SimHash fingerprint over normalized word/character 3-grams
/// for fast cross-source deduplication and story clustering.
pub fn simhash_64(text: &str) -> u64 {
    let normalized: String = text
        .to_ascii_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { ' ' })
        .collect();

    let tokens: Vec<&str> = normalized
        .split_whitespace()
        .filter(|w| w.len() > 2 && !is_stopword(w))
        .collect();

    if tokens.is_empty() {
        return 0;
    }

    let mut weights = [0i32; 64];
    for (idx, &tok) in tokens.iter().enumerate() {
        // Combine unigram and adjacent bigram features
        let h1 = fnv1a_64(tok.as_bytes());
        accumulate_bits(&mut weights, h1, 2);

        if idx + 1 < tokens.len() {
            let bigram = format!("{}_{}", tok, tokens[idx + 1]);
            let h2 = fnv1a_64(bigram.as_bytes());
            accumulate_bits(&mut weights, h2, 3);
        }
    }

    let mut fingerprint = 0u64;
    for (bit, &w) in weights.iter().enumerate() {
        if w > 0 {
            fingerprint |= 1u64 << bit;
        }
    }
    fingerprint
}

fn accumulate_bits(weights: &mut [i32; 64], hash: u64, weight: i32) {
    for bit in 0..64 {
        if (hash >> bit) & 1 == 1 {
            weights[bit] += weight;
        } else {
            weights[bit] -= weight;
        }
    }
}

fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub fn hamming_distance(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}

/// Computes token Jaccard similarity between two titles/abstracts (0.0 to 1.0).
pub fn token_jaccard_similarity(a: &str, b: &str) -> f64 {
    let set_a = significant_tokens(a);
    let set_b = significant_tokens(b);
    if set_a.is_empty() || set_b.is_empty() {
        return 0.0;
    }
    let intersection = set_a.intersection(&set_b).count() as f64;
    let union = set_a.union(&set_b).count() as f64;
    intersection / union
}

fn significant_tokens(text: &str) -> HashSet<String> {
    text.to_ascii_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| w.len() > 2 && !is_stopword(w))
        .map(|w| w.to_string())
        .collect()
}

fn is_stopword(w: &str) -> bool {
    matches!(
        w,
        "the"
            | "and"
            | "for"
            | "with"
            | "from"
            | "that"
            | "this"
            | "into"
            | "using"
            | "via"
            | "towards"
            | "new"
            | "how"
            | "why"
            | "what"
            | "over"
            | "under"
            | "between"
    )
}

/// Extracts an arXiv paper ID (e.g., `2610.12345`) from an arXiv URL or identifier,
/// and derives its direct PDF and `ar5iv` HTML5 links.
pub fn extract_arxiv_links(url: &str) -> Option<(String, String, String)> {
    let cleaned = url.trim().trim_end_matches('/');
    let id_part = if let Some(idx) = cleaned.find("arxiv.org/abs/") {
        &cleaned[idx + "arxiv.org/abs/".len()..]
    } else if let Some(idx) = cleaned.find("arxiv.org/pdf/") {
        cleaned[idx + "arxiv.org/pdf/".len()..].trim_end_matches(".pdf")
    } else {
        return None;
    };

    // Strip version suffix like v1/v2 for canonical ID if needed, but keep clean ID
    let paper_id = id_part.split('?').next().unwrap_or(id_part).trim();
    if paper_id.is_empty() {
        return None;
    }

    let abs_url = format!("https://arxiv.org/abs/{}", paper_id);
    let pdf_url = format!("https://arxiv.org/pdf/{}.pdf", paper_id);
    let ar5iv_url = format!("https://ar5iv.labs.arxiv.org/html/{}", paper_id);
    Some((abs_url, pdf_url, ar5iv_url))
}

/// Computes the deterministic Stage-1 BaseScore before LLM evaluation:
/// BaseScore = w1 * CrossSourceCount + w2 * ln(1 + CommunityPoints) + w3 * I(HasCodeRepo) + w4 * SourceWeight
pub fn compute_base_score(
    source_authority: f64,
    cross_source_count: u32,
    community_points: u32,
    has_code_repo: bool,
) -> f64 {
    let w1 = 2.4;
    let w2 = 1.35;
    let w3 = 2.0;
    let w4 = 1.0;

    let cross_term = w1 * ((cross_source_count.saturating_sub(1)) as f64);
    let comm_term = w2 * ((1.0 + community_points as f64).ln());
    let code_term = if has_code_repo { w3 } else { 0.0 };
    let auth_term = w4 * source_authority;

    ((cross_term + comm_term + code_term + auth_term) * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sponsor_filter() {
        assert!(is_sponsored_or_clickbait(
            "[Sponsored] Try CloudDB Today",
            "Fast SQL database",
            "https://example.com"
        ));
        assert!(is_sponsored_or_clickbait(
            "New Rust Compiler Backend",
            "Normal summary",
            "https://example.com/post?utm_medium=sponsored"
        ));
        assert!(!is_sponsored_or_clickbait(
            "Sub-linear Time Quantum Decoding for LDPC Codes",
            "We prove an $O(n \\log n)$ bound for quantum LDPC decoding.",
            "https://arxiv.org/abs/2610.01234"
        ));
    }

    #[test]
    fn test_latex_sanitizer() {
        let valid = "Runs in $O(n \\log n)$ time with $$E = mc^2$$.";
        assert_eq!(sanitize_and_validate_latex(valid), valid);

        let broken = "Costs $50 for $O(n)$ queries.";
        let fixed = sanitize_and_validate_latex(broken);
        assert_eq!(fixed, "Costs $50 for $O(n)\\$ queries.");
    }

    #[test]
    fn test_arxiv_link_extraction() {
        let (abs, pdf, ar5iv) =
            extract_arxiv_links("http://arxiv.org/abs/2610.12345v1").expect("valid arxiv url");
        assert_eq!(abs, "https://arxiv.org/abs/2610.12345v1");
        assert_eq!(pdf, "https://arxiv.org/pdf/2610.12345v1.pdf");
        assert_eq!(ar5iv, "https://ar5iv.labs.arxiv.org/html/2610.12345v1");
    }

    #[test]
    fn test_simhash_and_jaccard_clustering() {
        let a = "BitNet b1.58: 1-bit LLMs with Ternary Weights on CPU";
        let b = "BitNet b1.58 — Running 1-bit LLMs with Ternary Weights on CPUs";
        let c = "JWST Detects High-Redshift Galaxy Cluster in Early Universe";

        assert!(token_jaccard_similarity(a, b) > 0.45);
        assert!(token_jaccard_similarity(a, c) < 0.1);
        assert!(hamming_distance(simhash_64(a), simhash_64(b)) < 18);
    }
}
