use serde::{Deserialize, Serialize};

/// The 7 top-level scientific and technical areas in Ahbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Area {
    Cs,
    Ai,
    Math,
    Physics,
    Systems,
    Bio,
    Crypto,
}

impl Area {
    pub const ALL: [Area; 7] = [
        Area::Cs,
        Area::Ai,
        Area::Math,
        Area::Physics,
        Area::Systems,
        Area::Bio,
        Area::Crypto,
    ];

    pub fn slug(&self) -> &'static str {
        match self {
            Area::Cs => "cs",
            Area::Ai => "ai",
            Area::Math => "math",
            Area::Physics => "physics",
            Area::Systems => "systems",
            Area::Bio => "bio",
            Area::Crypto => "crypto",
        }
    }

    pub fn numeral(&self) -> &'static str {
        match self {
            Area::Cs => "I",
            Area::Ai => "II",
            Area::Math => "III",
            Area::Physics => "IV",
            Area::Systems => "V",
            Area::Bio => "VI",
            Area::Crypto => "VII",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Area::Cs => "Computer Science & Software",
            Area::Ai => "AI & Machine Learning",
            Area::Math => "Mathematics",
            Area::Physics => "Physics & Quantum",
            Area::Systems => "Systems, Hardware & Security",
            Area::Bio => "Computational Science & Bio",
            Area::Crypto => "Cryptography, Web3 & Blockchain",
        }
    }

    pub fn subtitle(&self) -> &'static str {
        match self {
            Area::Cs => "Languages, compilers, distributed systems, databases, web and open-source software.",
            Area::Ai => "Large language models, agents, learning theory, computer vision, robotics and inference.",
            Area::Math => "Number theory, combinatorics, logic, formal proof assistants (Lean/Rocq), algebra and analysis.",
            Area::Physics => "Quantum computing, condensed matter, astrophysics, high-energy theory and optics.",
            Area::Systems => "Operating systems, kernels, semiconductors, computer architecture, networking and InfoSec.",
            Area::Bio => "Bioinformatics, computational neuroscience, genomics, materials science and clean tech.",
            Area::Crypto => "Zero-knowledge proofs, cryptographic primitives, consensus protocols, smart contract security and MEV.",
        }
    }

    pub fn from_slug(s: &str) -> Option<Area> {
        match s.trim().to_ascii_lowercase().as_str() {
            "cs" | "computer-science" | "software" => Some(Area::Cs),
            "ai" | "ml" | "artificial-intelligence" => Some(Area::Ai),
            "math" | "mathematics" => Some(Area::Math),
            "physics" | "quantum" => Some(Area::Physics),
            "systems" | "hardware" | "security" => Some(Area::Systems),
            "bio" | "science" | "computational-science" => Some(Area::Bio),
            "crypto" | "cryptography" | "blockchain" | "web3" => Some(Area::Crypto),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EditionType {
    Daily,
    Weekend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusteredSource {
    pub source_name: String,
    pub title: String,
    pub url: String,
    pub points: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemLinks {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_ar5iv_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discussion_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discussion_label: Option<String>,
    #[serde(default)]
    pub clustered_sources: Vec<ClusteredSource>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TechnicalDetail {
    pub problem_context: String,
    pub method_architecture: String,
    pub findings_tradeoffs: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abstract_excerpt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bibtex: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemMetrics {
    pub base_score: f64,
    pub llm_novelty: f64,
    pub llm_rigor: f64,
    pub final_score: f64,
    pub community_points: u32,
    pub cross_source_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub id: String,
    pub issue_date: String,
    pub issue_number: u32,
    pub area: Area,
    pub is_top_pick: bool,
    pub is_paper: bool,
    /// 1-2 sentence sharp technical hook: what changed, was proved, or was built.
    pub headline: String,
    /// Single-paragraph concise technical summary (supports inline KaTeX).
    pub summary: String,
    /// Layer-2 technical analysis shown on the item detail page (?item=<id>).
    pub detail: TechnicalDetail,
    pub original_title: String,
    #[serde(default)]
    pub authors: Vec<String>,
    pub source_name: String,
    pub url: String,
    pub links: ItemLinks,
    pub tags: Vec<String>,
    pub metrics: ItemMetrics,
    pub published_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IssueStats {
    pub sources_checked: usize,
    pub sources_healthy: usize,
    pub raw_items_scanned: usize,
    pub clusters_formed: usize,
    pub qualified_items: usize,
    pub top_picks_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyIssue {
    pub issue_number: u32,
    pub date: String,
    pub edition_type: EditionType,
    pub generated_at: String,
    pub stats: IssueStats,
    pub items: Vec<NewsItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntry {
    pub issue_number: u32,
    pub date: String,
    pub edition_type: EditionType,
    pub qualified_items: usize,
    pub top_picks_count: usize,
    pub top_headlines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceHealthRecord {
    pub id: String,
    pub name: String,
    pub area: Area,
    pub url: String,
    pub status: String,
    pub items_fetched: usize,
    pub latency_ms: u64,
    pub last_checked: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DigestMode {
    Standard,
    Personalized,
    Hybrid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DigestDensity {
    Compact,
    Comprehensive,
}

impl DigestDensity {
    pub fn max_items(&self) -> usize {
        match self {
            DigestDensity::Compact => 7,
            DigestDensity::Comprehensive => 18,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DigestFrequency {
    Daily,
    WeekdaysOnly,
    WeekendOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContentFilter {
    All,
    PapersOnly,
    CodeOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionPreferences {
    pub email: String,
    pub mode: DigestMode,
    pub density: DigestDensity,
    pub frequency: DigestFrequency,
    pub content_filter: ContentFilter,
    pub areas: Vec<Area>,
    pub tags: Vec<String>,
    #[serde(default)]
    pub verified: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignalVote {
    Signal,
    Noise,
}
