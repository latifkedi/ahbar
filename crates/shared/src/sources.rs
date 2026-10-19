use crate::models::Area;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
    ArxivApi,
    RssAtom,
    HackerNewsApi,
    HuggingFacePapersApi,
}

#[derive(Debug, Clone)]
pub struct SourceDescriptor {
    pub id: &'static str,
    pub name: &'static str,
    pub area: Area,
    pub kind: SourceKind,
    pub url: &'static str,
    pub authority_weight: f64,
    pub is_academic_paper: bool,
    pub default_tags: &'static [&'static str],
}

/// Curated primary sources across the 7 areas of Ahbar.
pub const SOURCES: &[SourceDescriptor] = &[
    // =========================================================================
    // I. Computer Science & Software Engineering
    // =========================================================================
    SourceDescriptor {
        id: "arxiv-cs-pl-se",
        name: "arXiv CS (PL, SE, DB, DC)",
        area: Area::Cs,
        kind: SourceKind::ArxivApi,
        url: "https://export.arxiv.org/api/query?search_query=cat:cs.PL+OR+cat:cs.SE+OR+cat:cs.DB+OR+cat:cs.DC+OR+cat:cs.DS&sortBy=submittedDate&sortOrder=descending&max_results=30",
        authority_weight: 8.8,
        is_academic_paper: true,
        default_tags: &["compilers", "distributed-systems"],
    },
    SourceDescriptor {
        id: "hacker-news-top",
        name: "Hacker News",
        area: Area::Cs,
        kind: SourceKind::HackerNewsApi,
        url: "https://hn.algolia.com/api/v1/search?tags=front_page&hitsPerPage=35",
        authority_weight: 8.2,
        is_academic_paper: false,
        default_tags: &["software-architecture", "open-source"],
    },
    SourceDescriptor {
        id: "lobsters",
        name: "Lobste.rs",
        area: Area::Cs,
        kind: SourceKind::RssAtom,
        url: "https://lobste.rs/rss",
        authority_weight: 8.5,
        is_academic_paper: false,
        default_tags: &["rust", "compilers"],
    },
    SourceDescriptor {
        id: "acm-technews",
        name: "Communications of the ACM",
        area: Area::Cs,
        kind: SourceKind::RssAtom,
        url: "https://cacm.acm.org/feed/",
        authority_weight: 8.4,
        is_academic_paper: false,
        default_tags: &["algorithms", "software-architecture"],
    },

    // =========================================================================
    // II. AI & Machine Learning
    // =========================================================================
    SourceDescriptor {
        id: "hf-daily-papers",
        name: "Hugging Face Daily Papers",
        area: Area::Ai,
        kind: SourceKind::HuggingFacePapersApi,
        url: "https://huggingface.co/api/daily_papers",
        authority_weight: 9.4,
        is_academic_paper: true,
        default_tags: &["llm", "open-weights"],
    },
    SourceDescriptor {
        id: "arxiv-ai-ml",
        name: "arXiv AI & ML (cs.AI, cs.LG, cs.CL, cs.CV)",
        area: Area::Ai,
        kind: SourceKind::ArxivApi,
        url: "https://export.arxiv.org/api/query?search_query=cat:cs.AI+OR+cat:cs.LG+OR+cat:cs.CL+OR+cat:cs.CV+OR+cat:cs.RO&sortBy=submittedDate&sortOrder=descending&max_results=35",
        authority_weight: 8.9,
        is_academic_paper: true,
        default_tags: &["llm", "ml-theory"],
    },

    // =========================================================================
    // III. Mathematics
    // =========================================================================
    SourceDescriptor {
        id: "arxiv-math",
        name: "arXiv Mathematics (NT, CO, AG, LO, PR)",
        area: Area::Math,
        kind: SourceKind::ArxivApi,
        url: "https://export.arxiv.org/api/query?search_query=cat:math.NT+OR+cat:math.CO+OR+cat:math.AG+OR+cat:math.LO+OR+cat:math.PR+OR+cat:math.AP&sortBy=submittedDate&sortOrder=descending&max_results=30",
        authority_weight: 9.2,
        is_academic_paper: true,
        default_tags: &["number-theory", "combinatorics"],
    },
    SourceDescriptor {
        id: "quanta-math",
        name: "Quanta Magazine (Mathematics)",
        area: Area::Math,
        kind: SourceKind::RssAtom,
        url: "https://api.quantamagazine.org/v1/izar/feed/?section=mathematics",
        authority_weight: 9.3,
        is_academic_paper: false,
        default_tags: &["number-theory", "logic"],
    },
    SourceDescriptor {
        id: "terence-tao",
        name: "What's New (Terence Tao)",
        area: Area::Math,
        kind: SourceKind::RssAtom,
        url: "https://terrytao.wordpress.com/feed/",
        authority_weight: 9.6,
        is_academic_paper: false,
        default_tags: &["analysis-pde", "formal-proofs"],
    },

    // =========================================================================
    // IV. Physics & Quantum
    // =========================================================================
    SourceDescriptor {
        id: "arxiv-physics-quantum",
        name: "arXiv Physics (quant-ph, cond-mat, hep-th, astro-ph)",
        area: Area::Physics,
        kind: SourceKind::ArxivApi,
        url: "https://export.arxiv.org/api/query?search_query=cat:quant-ph+OR+cat:cond-mat.supr-con+OR+cat:hep-th+OR+cat:astro-ph.CO+OR+cat:physics.optics&sortBy=submittedDate&sortOrder=descending&max_results=30",
        authority_weight: 9.1,
        is_academic_paper: true,
        default_tags: &["quantum-computing", "condensed-matter"],
    },
    SourceDescriptor {
        id: "aps-physics",
        name: "APS Physics Magazine",
        area: Area::Physics,
        kind: SourceKind::RssAtom,
        url: "https://physics.aps.org/rss/recent.xml",
        authority_weight: 9.2,
        is_academic_paper: false,
        default_tags: &["quantum-physics", "condensed-matter"],
    },
    SourceDescriptor {
        id: "nature-physics",
        name: "Nature Physics",
        area: Area::Physics,
        kind: SourceKind::RssAtom,
        url: "https://www.nature.com/nphys.rss",
        authority_weight: 9.4,
        is_academic_paper: true,
        default_tags: &["quantum-physics", "optics-photonics"],
    },

    // =========================================================================
    // V. Systems, Hardware & Security
    // =========================================================================
    SourceDescriptor {
        id: "arxiv-systems-arch",
        name: "arXiv Systems & Architecture (cs.OS, cs.AR, cs.NI)",
        area: Area::Systems,
        kind: SourceKind::ArxivApi,
        url: "https://export.arxiv.org/api/query?search_query=cat:cs.OS+OR+cat:cs.AR+OR+cat:cs.NI+OR+cat:cs.PF&sortBy=submittedDate&sortOrder=descending&max_results=25",
        authority_weight: 8.8,
        is_academic_paper: true,
        default_tags: &["computer-architecture", "kernel-os"],
    },
    SourceDescriptor {
        id: "lwn-net",
        name: "LWN.net (Linux & Kernel)",
        area: Area::Systems,
        kind: SourceKind::RssAtom,
        url: "https://lwn.net/headlines/rss",
        authority_weight: 9.1,
        is_academic_paper: false,
        default_tags: &["kernel-os", "performance"],
    },
    SourceDescriptor {
        id: "project-zero",
        name: "Google Project Zero",
        area: Area::Systems,
        kind: SourceKind::RssAtom,
        url: "https://googleprojectzero.blogspot.com/feeds/posts/default?alt=rss",
        authority_weight: 9.5,
        is_academic_paper: false,
        default_tags: &["infosec", "kernel-os"],
    },
    SourceDescriptor {
        id: "trail-of-bits",
        name: "Trail of Bits Security Research",
        area: Area::Systems,
        kind: SourceKind::RssAtom,
        url: "https://blog.trailofbits.com/feed/",
        authority_weight: 9.0,
        is_academic_paper: false,
        default_tags: &["infosec", "cryptography"],
    },

    // =========================================================================
    // VI. Computational Science & Bio
    // =========================================================================
    SourceDescriptor {
        id: "arxiv-qbio",
        name: "arXiv Quantitative Biology (q-bio)",
        area: Area::Bio,
        kind: SourceKind::ArxivApi,
        url: "https://export.arxiv.org/api/query?search_query=cat:q-bio.QM+OR+cat:q-bio.GN+OR+cat:q-bio.NC+OR+cat:q-bio.BM+OR+cat:cond-mat.mtrl-sci&sortBy=submittedDate&sortOrder=descending&max_results=25",
        authority_weight: 8.9,
        is_academic_paper: true,
        default_tags: &["bioinformatics", "genomics-proteins"],
    },
    SourceDescriptor {
        id: "nature-bt-comp",
        name: "Nature Computational Science",
        area: Area::Bio,
        kind: SourceKind::RssAtom,
        url: "https://www.nature.com/natcomputsci.rss",
        authority_weight: 9.4,
        is_academic_paper: true,
        default_tags: &["bioinformatics", "materials-science"],
    },

    // =========================================================================
    // VII. Cryptography, Web3 & Blockchain Systems
    // =========================================================================
    SourceDescriptor {
        id: "iacr-eprint",
        name: "IACR Cryptology ePrint Archive",
        area: Area::Crypto,
        kind: SourceKind::RssAtom,
        url: "https://eprint.iacr.org/rss/rss.xml",
        authority_weight: 9.5,
        is_academic_paper: true,
        default_tags: &["cryptography", "zkp"],
    },
    SourceDescriptor {
        id: "arxiv-crypto",
        name: "arXiv Cryptography & Security (cs.CR)",
        area: Area::Crypto,
        kind: SourceKind::ArxivApi,
        url: "https://export.arxiv.org/api/query?search_query=cat:cs.CR&sortBy=submittedDate&sortOrder=descending&max_results=25",
        authority_weight: 8.9,
        is_academic_paper: true,
        default_tags: &["cryptography", "zkp"],
    },
    SourceDescriptor {
        id: "ethresearch",
        name: "Ethereum Research (ethresear.ch)",
        area: Area::Crypto,
        kind: SourceKind::RssAtom,
        url: "https://ethresear.ch/latest.rss",
        authority_weight: 8.7,
        is_academic_paper: false,
        default_tags: &["zkp", "consensus"],
    },
    SourceDescriptor {
        id: "vitalik-blog",
        name: "Vitalik Buterin's Blog",
        area: Area::Crypto,
        kind: SourceKind::RssAtom,
        url: "https://vitalik.eth.limo/feed.xml",
        authority_weight: 9.2,
        is_academic_paper: false,
        default_tags: &["zkp", "consensus"],
    },
];
