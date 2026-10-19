use serde::{Deserialize, Serialize};

/// The 10 top-level scientific, technical, economic, geopolitical, and philosophical areas in Ahbar.
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
    Econ,
    Geopolitics,
    Philosophy,
}

impl Area {
    pub const ALL: [Area; 10] = [
        Area::Cs,
        Area::Ai,
        Area::Math,
        Area::Physics,
        Area::Systems,
        Area::Bio,
        Area::Crypto,
        Area::Econ,
        Area::Geopolitics,
        Area::Philosophy,
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
            Area::Econ => "econ",
            Area::Geopolitics => "geopolitics",
            Area::Philosophy => "philosophy",
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
            Area::Econ => "VIII",
            Area::Geopolitics => "IX",
            Area::Philosophy => "X",
        }
    }

    pub fn short_label(&self, is_tr: bool) -> &'static str {
        match (self, is_tr) {
            (Area::Cs, false) => "CS",
            (Area::Cs, true) => "Yazılım",
            (Area::Ai, false) => "AI",
            (Area::Ai, true) => "YZ",
            (Area::Math, false) => "Math",
            (Area::Math, true) => "Matematik",
            (Area::Physics, false) => "Physics",
            (Area::Physics, true) => "Fizik",
            (Area::Systems, false) => "Systems",
            (Area::Systems, true) => "Sistem",
            (Area::Bio, false) => "Bio",
            (Area::Bio, true) => "Biyo",
            (Area::Crypto, false) => "Crypto",
            (Area::Crypto, true) => "Kripto",
            (Area::Econ, false) => "Econ",
            (Area::Econ, true) => "Ekonomi",
            (Area::Geopolitics, false) => "IR & Geo",
            (Area::Geopolitics, true) => "Jeopolitik",
            (Area::Philosophy, false) => "Phil & Cog",
            (Area::Philosophy, true) => "Felsefe",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Area::Cs => "Computer Science & Software",
            Area::Ai => "AI & Machine Learning",
            Area::Math => "Mathematics & Formal Sciences",
            Area::Physics => "Physics, Quantum & Space",
            Area::Systems => "Systems, Hardware & Security",
            Area::Bio => "Biotech, Medicine & Materials",
            Area::Crypto => "Cryptography, Web3 & Protocols",
            Area::Econ => "Economics, Markets & Quantitative Finance",
            Area::Geopolitics => "International Relations, Geopolitics & Strategy",
            Area::Philosophy => "Philosophy of Science, Decision Theory & Cognition",
        }
    }

    pub fn title_tr(&self) -> &'static str {
        match self {
            Area::Cs => "Bilgisayar Bilimi & Yazılım",
            Area::Ai => "Yapay Zekâ & Makine Öğrenmesi",
            Area::Math => "Matematik & Biçimsel Bilimler",
            Area::Physics => "Fizik, Kuantum & Uzay",
            Area::Systems => "Sistem, Donanım & Siber Güvenlik",
            Area::Bio => "Biyoteknoloji, Tıp & Malzeme Bilimi",
            Area::Crypto => "Kriptografi, Web3 & Dağıtık Protokoller",
            Area::Econ => "Ekonomi, Finans & Makro Analiz",
            Area::Geopolitics => "Uluslararası İlişkiler, Jeopolitik & Strateji",
            Area::Philosophy => "Bilim Felsefesi, Karar Kuramı & Bilişsel Bilimler",
        }
    }

    pub fn subtitle(&self) -> &'static str {
        match self {
            Area::Cs => "Languages, compilers, distributed systems, databases, formal verification and open-source software.",
            Area::Ai => "Large language models, autonomous agents, learning theory, computer vision, robotics and inference.",
            Area::Math => "Number theory, combinatorics, logic, formal proof assistants (Lean 4 / Rocq), algebra and analysis.",
            Area::Physics => "Quantum computing, condensed matter, astrophysics, high-energy theory and optics.",
            Area::Systems => "Operating systems, kernels, semiconductors, computer architecture, networking and InfoSec.",
            Area::Bio => "Bioinformatics, computational neuroscience, genomics, protein design, materials science and clean tech.",
            Area::Crypto => "Zero-knowledge proofs, post-quantum cryptography, consensus protocols, smart contract security and MEV.",
            Area::Econ => "Macroeconomics, monetary policy, econometrics, quantitative finance, market design and industrial organization.",
            Area::Geopolitics => "Grand strategy, technology & semiconductor policy, defense analysis, energy security and international law.",
            Area::Philosophy => "Philosophy of science, formal epistemology, decision & game theory, cognitive science and complex systems.",
        }
    }

    pub fn subtitle_tr(&self) -> &'static str {
        match self {
            Area::Cs => "Programlama dilleri, derleyiciler, dağıtık sistemler, veritabanları, biçimsel doğrulama ve açık kaynak.",
            Area::Ai => "Büyük dil modelleri, otonom ajanlar, öğrenme kuramı, bilgisayarlı görü, robotik ve çıkarım motorları.",
            Area::Math => "Sayılar kuramı, kombinatorik, mantık, biçimsel kanıt asistanları (Lean 4 / Rocq), cebir ve analiz.",
            Area::Physics => "Kuantum hesaplama, yoğun madde, astrofizik, yüksek enerji kuramı ve optik/fotonik.",
            Area::Systems => "İşletim sistemleri, çekirdek, yarı iletkenler, bilgisayar mimarisi, ağ protokolleri ve siber güvenlik.",
            Area::Bio => "Biyoinformatik, hesaplamalı sinirbilim, genomik, protein tasarımı, malzeme bilimi ve temiz enerji.",
            Area::Crypto => "Sıfır-bilgi kanıtları (ZKP), kuantum-sonrası kriptografi, mutabakat protokolleri ve akıllı sözleşme güvenliği.",
            Area::Econ => "Makroekonomi, para politikası, ekonometri, kantitatif finans, piyasa tasarımı ve sanayi iktisadı.",
            Area::Geopolitics => "Küresel strateji, teknoloji ve çip jeopolitiği, savunma analizi, enerji güvenliği ve uluslararası hukuk.",
            Area::Philosophy => "Bilim felsefesi, biçimsel epistemoloji, karar ve oyun kuramı, bilişsel bilimler ve karmaşık sistemler.",
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
            "econ" | "economics" | "finance" | "q-fin" => Some(Area::Econ),
            "geopolitics" | "ir" | "strategy" | "international-relations" => Some(Area::Geopolitics),
            "philosophy" | "cognition" | "decision-theory" | "phil" => Some(Area::Philosophy),
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub problem_context_tr: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method_architecture_tr: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub findings_tradeoffs_tr: Option<String>,
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
    /// 1-2 sentence sharp technical hook: what changed, was proved, or was built (English).
    pub headline: String,
    /// Single-paragraph concise technical summary (English).
    pub summary: String,
    /// 1-2 sentence sharp technical hook in Turkish.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headline_tr: Option<String>,
    /// Single-paragraph concise technical summary in Turkish.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary_tr: Option<String>,
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
    /// 1 item per area (10 total) for ultra-fast scan
    Compact,
    /// 3 items per area (30 total across 10 areas) — Recommended default
    #[serde(alias = "three_per_area")]
    ThreePerArea,
    /// 5 items per area (50 total) for deep reading
    Comprehensive,
}

impl DigestDensity {
    pub fn items_per_area(&self) -> usize {
        match self {
            DigestDensity::Compact => 1,
            DigestDensity::ThreePerArea => 3,
            DigestDensity::Comprehensive => 5,
        }
    }

    pub fn max_items(&self) -> usize {
        self.items_per_area() * Area::ALL.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DigestFrequency {
    Daily,
    #[serde(alias = "weekdays_only")]
    WeekdaysOnly,
    #[serde(alias = "weekend_only")]
    WeekendOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContentFilter {
    All,
    #[serde(alias = "papers_only")]
    PapersOnly,
    #[serde(alias = "code_only")]
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
    #[serde(default = "default_lang")]
    pub lang: String,
    #[serde(default)]
    pub verified: bool,
}

fn default_lang() -> String {
    "tr".to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignalVote {
    Signal,
    Noise,
}
