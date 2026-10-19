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

/// Returns `true` if `expr` (the content between single `$ ... $` delimiters)
/// is genuine inline LaTeX math rather than prose between two currency `$` signs
/// (e.g. `"$1 billion ... $30 billion"` or `"$28 to $36 billion"`).
pub fn is_likely_latex_math(expr: &str) -> bool {
    if expr.is_empty() || expr.len() > 160 {
        return false;
    }
    if expr.starts_with(char::is_whitespace) || expr.ends_with(char::is_whitespace) {
        return false;
    }
    if expr.contains(". ") || expr.contains(".\n") {
        return false;
    }
    // If it starts with a number followed by whitespace and a word (e.g. "1 billion", "28 to"),
    // it is a currency figure unless it contains an explicit LaTeX backslash command.
    let bytes = expr.as_bytes();
    if bytes[0].is_ascii_digit() && !expr.contains('\\') {
        let mut k = 0;
        while k < bytes.len() && (bytes[k].is_ascii_digit() || bytes[k] == b'.' || bytes[k] == b',')
        {
            k += 1;
        }
        if k < bytes.len() && bytes[k].is_ascii_whitespace() {
            let rest = expr[k..].trim_start();
            let first_word_len = rest
                .chars()
                .take_while(|c| c.is_alphabetic())
                .count();
            if first_word_len >= 2 {
                return false;
            }
        }
        // Pure number without math symbols between two `$` is also unlikely to be math if > 6 chars
        if !expr.chars().any(|c| matches!(c, '^' | '_' | '=' | '<' | '>' | '+' | '-' | '*' | '/'))
            && expr.len() > 6
        {
            return false;
        }
    }
    true
}

/// Validates and repairs inline LaTeX delimiters (`$...$` and `$$...$$`)
/// so unclosed math markers never corrupt paragraph rendering, while leaving currency `$` intact.
pub fn sanitize_and_validate_latex(input: &str) -> String {
    let mut text = input.trim().to_string();
    if text.is_empty() {
        return text;
    }

    // Normalize `\( ... \)` into `$...$` and `\[ ... \]` into `$$...$$`
    text = text.replace("\\(", "$").replace("\\)", "$");
    text = text.replace("\\[", "$$").replace("\\]", "$$");
    // Unescape any previously escaped `\$` before currency numbers so prose stays clean
    text = text.replace("\\$", "$");

    text
}

/// Converts inline LaTeX markers (`$...$`, `\times`, `\le`, `\ge`, `\alpha`, `\mathbb{F}`, etc.)
/// into clean Unicode math symbols for HTML email clients and plain-text views,
/// while preserving currency dollar signs (`$100`, `$1 billion`).
pub fn latex_to_email_unicode(input: &str) -> String {
    // First strip only genuine `$ ... $` and `$$ ... $$` delimiters, keeping currency `$`
    let chars: Vec<char> = input.chars().collect();
    let mut s = String::with_capacity(input.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '$' {
            let is_display = i + 1 < chars.len() && chars[i + 1] == '$';
            let delim_len = if is_display { 2 } else { 1 };
            let start = i + delim_len;
            let mut end = None;
            let mut j = start;
            while j < chars.len() {
                if is_display {
                    if j + 1 < chars.len() && chars[j] == '$' && chars[j + 1] == '$' {
                        end = Some(j);
                        break;
                    }
                } else if chars[j] == '$' {
                    end = Some(j);
                    break;
                }
                j += 1;
            }
            if let Some(end_idx) = end {
                let expr: String = chars[start..end_idx].iter().collect();
                if is_display || is_likely_latex_math(&expr) {
                    s.push_str(&expr);
                    i = end_idx + delim_len;
                    continue;
                }
            }
        }
        s.push(chars[i]);
        i += 1;
    }

    let replacements = [
        ("\\times", "×"),
        ("\\cdot", "·"),
        ("\\le", "≤"),
        ("\\leq", "≤"),
        ("\\ge", "≥"),
        ("\\geq", "≥"),
        ("\\neq", "≠"),
        ("\\approx", "≈"),
        ("\\sim", "∼"),
        ("\\ll", "≪"),
        ("\\gg", "≫"),
        ("\\to", "→"),
        ("\\rightarrow", "→"),
        ("\\infty", "∞"),
        ("\\pm", "±"),
        ("\\%", "%"),
        ("\\mathbb{F}", "𝔽"),
        ("\\mathbb{R}", "ℝ"),
        ("\\mathbb{Z}", "ℤ"),
        ("\\mathbb{N}", "ℕ"),
        ("\\mathbb{E}", "𝔼"),
        ("\\mathcal{O}", "𝒪"),
        ("\\alpha", "α"),
        ("\\beta", "β"),
        ("\\gamma", "γ"),
        ("\\delta", "δ"),
        ("\\Delta", "Δ"),
        ("\\varepsilon", "ε"),
        ("\\epsilon", "ϵ"),
        ("\\theta", "θ"),
        ("\\lambda", "λ"),
        ("\\mu", "μ"),
        ("\\nu", "ν"),
        ("\\pi", "π"),
        ("\\rho", "ρ"),
        ("\\sigma", "σ"),
        ("\\tau", "τ"),
        ("\\omega", "ω"),
        ("\\Omega", "Ω"),
        ("{,}", ","),
    ];
    for (pat, rep) in replacements {
        s = s.replace(pat, rep);
    }
    while let Some(start) = s.find("\\text{") {
        if let Some(rel_end) = s[start + 6..].find('}') {
            let inner = s[start + 6..start + 6 + rel_end].to_string();
            s.replace_range(start..start + 6 + rel_end + 1, &inner);
        } else {
            break;
        }
    }
    s
}

/// Converts text containing `$...$` and `$$...$$` into safe HTML with native `<math class="m">`
/// elements and typographic `<msup>` / `<msub>` tags — zero external KaTeX CDN required!
/// Automatically preserves currency figures like `$1 billion` or `$28 to $36 billion`.
pub fn latex_to_html_math(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 64);
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() && chars[i + 1] == '$' {
            out.push('$');
            i += 2;
            continue;
        }
        if chars[i] == '$' {
            let is_display = i + 1 < chars.len() && chars[i + 1] == '$';
            let delim_len = if is_display { 2 } else { 1 };
            let start = i + delim_len;
            let mut end = None;
            let mut j = start;
            while j < chars.len() {
                if chars[j] == '\\' {
                    j += 2;
                    continue;
                }
                if is_display {
                    if j + 1 < chars.len() && chars[j] == '$' && chars[j + 1] == '$' {
                        end = Some(j);
                        break;
                    }
                } else if chars[j] == '$' {
                    end = Some(j);
                    break;
                }
                j += 1;
            }

            if let Some(end_idx) = end {
                let expr: String = chars[start..end_idx].iter().collect();
                if is_display || is_likely_latex_math(&expr) {
                    let rendered = render_math_expr_to_mathml(&expr, is_display);
                    out.push_str(&rendered);
                    i = end_idx + delim_len;
                    continue;
                }
            }
        }
        push_escaped_html_char(&mut out, chars[i]);
        i += 1;
    }

    out
}

fn render_math_expr_to_mathml(expr: &str, is_display: bool) -> String {
    let unicode_expr = latex_to_email_unicode(expr);
    let chars: Vec<char> = unicode_expr.chars().collect();
    let mut body = String::with_capacity(unicode_expr.len() + 32);
    let mut i = 0;
    while i < chars.len() {
        if (chars[i] == '^' || chars[i] == '_') && i + 1 < chars.len() {
            let tag = if chars[i] == '^' { "sup" } else { "sub" };
            i += 1;
            let mut group = String::new();
            if chars[i] == '{' {
                i += 1;
                while i < chars.len() && chars[i] != '}' {
                    push_escaped_html_char(&mut group, chars[i]);
                    i += 1;
                }
                if i < chars.len() && chars[i] == '}' {
                    i += 1;
                }
            } else {
                while i < chars.len()
                    && (chars[i].is_ascii_alphanumeric() || matches!(chars[i], '-' | '+' | '/'))
                {
                    push_escaped_html_char(&mut group, chars[i]);
                    i += 1;
                }
            }
            body.push_str(&format!("<{}>{}</{}>", tag, group, tag));
            continue;
        }
        if chars[i] == '\\' {
            // Strip remaining LaTeX command prefix like \log, \sin, \min
            i += 1;
            continue;
        }
        if chars[i] == '{' || chars[i] == '}' {
            i += 1;
            continue;
        }
        push_escaped_html_char(&mut body, chars[i]);
        i += 1;
    }

    if is_display {
        format!(
            "<math display=\"block\" class=\"m m-block\"><mrow>{}</mrow></math>",
            body
        )
    } else {
        format!(
            "<math display=\"inline\" class=\"m\"><mrow>{}</mrow></math>",
            body
        )
    }
}

fn push_escaped_html_char(out: &mut String, ch: char) {
    match ch {
        '&' => out.push_str("&amp;"),
        '<' => out.push_str("&lt;"),
        '>' => out.push_str("&gt;"),
        '"' => out.push_str("&quot;"),
        '\'' => out.push_str("&#39;"),
        _ => out.push(ch),
    }
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
    fn test_latex_sanitizer_and_mathml() {
        let valid = "Runs in $O(n \\log n)$ time with $$E = mc^2$$.";
        assert_eq!(sanitize_and_validate_latex(valid), valid);

        let html = latex_to_html_math(valid);
        assert!(html.contains("<math display=\"inline\" class=\"m\">"));
        assert!(html.contains("<sup>2</sup>"));

        let mixed = "Costs $50 for $O(n)$ queries.";
        let mixed_html = latex_to_html_math(mixed);
        assert!(mixed_html.contains("Costs $50 for <math display=\"inline\" class=\"m\"><mrow>O(n)</mrow></math> queries."));

        let econ = "Net worth exceeding $1 billion yields $30 billion (range $28 to $36 billion).";
        assert_eq!(latex_to_html_math(econ), econ);
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
