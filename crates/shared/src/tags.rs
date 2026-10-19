use std::collections::HashSet;

/// Canonical tag entry with display label and primary area slug.
#[derive(Debug, Clone, Copy)]
pub struct CanonicalTag {
    pub slug: &'static str,
    pub label: &'static str,
    pub area_slug: &'static str,
}

/// Canonical tag dictionary (inspired by useful-sites/data/tags.py).
/// Free-form tags emitted by feeds or LLMs are normalized into this fixed set
/// so the filter bar stays clean and free of single-use noise.
pub const CANON_TAGS: &[CanonicalTag] = &[
    // I. Computer Science & Software
    CanonicalTag { slug: "rust", label: "Rust", area_slug: "cs" },
    CanonicalTag { slug: "c-cpp", label: "C & C++", area_slug: "cs" },
    CanonicalTag { slug: "go", label: "Go", area_slug: "cs" },
    CanonicalTag { slug: "python", label: "Python", area_slug: "cs" },
    CanonicalTag { slug: "typescript", label: "TypeScript & Web", area_slug: "cs" },
    CanonicalTag { slug: "compilers", label: "Compilers & PL", area_slug: "cs" },
    CanonicalTag { slug: "databases", label: "Databases", area_slug: "cs" },
    CanonicalTag { slug: "distributed-systems", label: "Distributed Systems", area_slug: "cs" },
    CanonicalTag { slug: "algorithms", label: "Algorithms", area_slug: "cs" },
    CanonicalTag { slug: "open-source", label: "Open Source", area_slug: "cs" },
    CanonicalTag { slug: "software-architecture", label: "Architecture", area_slug: "cs" },

    // II. AI & Machine Learning
    CanonicalTag { slug: "llm", label: "LLMs", area_slug: "ai" },
    CanonicalTag { slug: "agents", label: "Agents", area_slug: "ai" },
    CanonicalTag { slug: "inference", label: "Inference & Quantization", area_slug: "ai" },
    CanonicalTag { slug: "rag", label: "RAG & Retrieval", area_slug: "ai" },
    CanonicalTag { slug: "ml-theory", label: "ML Theory", area_slug: "ai" },
    CanonicalTag { slug: "computer-vision", label: "Computer Vision", area_slug: "ai" },
    CanonicalTag { slug: "robotics", label: "Robotics", area_slug: "ai" },
    CanonicalTag { slug: "multimodal", label: "Multimodal & Audio", area_slug: "ai" },
    CanonicalTag { slug: "reinforcement-learning", label: "Reinforcement Learning", area_slug: "ai" },
    CanonicalTag { slug: "open-weights", label: "Open Weights", area_slug: "ai" },

    // III. Mathematics
    CanonicalTag { slug: "number-theory", label: "Number Theory", area_slug: "math" },
    CanonicalTag { slug: "combinatorics", label: "Combinatorics & Graph Theory", area_slug: "math" },
    CanonicalTag { slug: "logic", label: "Logic & Foundations", area_slug: "math" },
    CanonicalTag { slug: "formal-proofs", label: "Formal Proofs (Lean/Rocq)", area_slug: "math" },
    CanonicalTag { slug: "algebra-geometry", label: "Algebra & Geometry", area_slug: "math" },
    CanonicalTag { slug: "analysis-pde", label: "Analysis & PDEs", area_slug: "math" },
    CanonicalTag { slug: "probability-stats", label: "Probability & Statistics", area_slug: "math" },
    CanonicalTag { slug: "topology", label: "Topology", area_slug: "math" },
    CanonicalTag { slug: "optimization", label: "Optimization", area_slug: "math" },

    // IV. Physics & Quantum
    CanonicalTag { slug: "quantum-computing", label: "Quantum Computing", area_slug: "physics" },
    CanonicalTag { slug: "quantum-physics", label: "Quantum Mechanics", area_slug: "physics" },
    CanonicalTag { slug: "condensed-matter", label: "Condensed Matter", area_slug: "physics" },
    CanonicalTag { slug: "astrophysics", label: "Astrophysics & Cosmology", area_slug: "physics" },
    CanonicalTag { slug: "high-energy", label: "High-Energy & Particle Physics", area_slug: "physics" },
    CanonicalTag { slug: "optics-photonics", label: "Optics & Photonics", area_slug: "physics" },
    CanonicalTag { slug: "fluid-dynamics", label: "Fluid & Statistical Mechanics", area_slug: "physics" },

    // V. Systems, Hardware & Security
    CanonicalTag { slug: "kernel-os", label: "Kernel & OS", area_slug: "systems" },
    CanonicalTag { slug: "semiconductors", label: "Semiconductors & Chips", area_slug: "systems" },
    CanonicalTag { slug: "computer-architecture", label: "Computer Architecture", area_slug: "systems" },
    CanonicalTag { slug: "infosec", label: "Vulnerability & InfoSec", area_slug: "systems" },
    CanonicalTag { slug: "networking", label: "Networking & Protocols", area_slug: "systems" },
    CanonicalTag { slug: "embedded", label: "Embedded & Firmware", area_slug: "systems" },
    CanonicalTag { slug: "performance", label: "Performance & Profiling", area_slug: "systems" },

    // VI. Computational Science & Bio
    CanonicalTag { slug: "bioinformatics", label: "Bioinformatics", area_slug: "bio" },
    CanonicalTag { slug: "genomics-proteins", label: "Genomics & Protein Folding", area_slug: "bio" },
    CanonicalTag { slug: "neuroscience", label: "Computational Neuroscience", area_slug: "bio" },
    CanonicalTag { slug: "materials-science", label: "Materials Science", area_slug: "bio" },
    CanonicalTag { slug: "clean-tech", label: "Energy & Fusion", area_slug: "bio" },
    CanonicalTag { slug: "climate-earth", label: "Climate & Earth Simulation", area_slug: "bio" },

    // VII. Cryptography, Web3 & Blockchain
    CanonicalTag { slug: "cryptography", label: "Cryptography", area_slug: "crypto" },
    CanonicalTag { slug: "zkp", label: "Zero-Knowledge Proofs", area_slug: "crypto" },
    CanonicalTag { slug: "post-quantum", label: "Post-Quantum Crypto", area_slug: "crypto" },
    CanonicalTag { slug: "consensus", label: "Consensus & BFT", area_slug: "crypto" },
    CanonicalTag { slug: "smart-contracts", label: "Smart Contracts & EVM", area_slug: "crypto" },
    CanonicalTag { slug: "mev-defi", label: "MEV & Protocol Economics", area_slug: "crypto" },
];

/// Maps raw/free-form tags or arXiv categories into canonical Ahbar tag slugs.
pub fn resolve_canonical_tag(raw: &str) -> Option<&'static str> {
    let cleaned = raw.trim().to_ascii_lowercase().replace(['_', ' '], "-");
    let s = cleaned.as_str();

    // Direct match against CANON_TAGS
    for t in CANON_TAGS {
        if t.slug == s {
            return Some(t.slug);
        }
    }

    // Alias mapping table
    match s {
        // CS & Software aliases
        "c" | "cpp" | "c++" | "clang" | "cmake" => Some("c-cpp"),
        "golang" => Some("go"),
        "js" | "javascript" | "ts" | "web" | "frontend" | "browser" | "wasm" | "webassembly" => {
            Some("typescript")
        }
        "compiler" | "llvm" | "programming-languages" | "cs.pl" | "type-systems" | "jit" => {
            Some("compilers")
        }
        "db" | "database" | "sql" | "postgres" | "postgresql" | "sqlite" | "cs.db" | "storage" => {
            Some("databases")
        }
        "distributed" | "cs.dc" | "raft" | "paxos" | "cloud" | "kubernetes" => {
            Some("distributed-systems")
        }
        "algorithm" | "cs.ds" | "data-structures" | "complexity" | "cs.cc" => Some("algorithms"),
        "oss" | "github" | "mit" | "apache" | "foss" => Some("open-source"),
        "cs.se" | "software-engineering" | "system-design" => Some("software-architecture"),

        // AI & ML aliases
        "llms" | "large-language-models" | "gpt" | "transformer" | "transformers" | "cs.cl"
        | "nlp" => Some("llm"),
        "agent" | "multi-agent" | "agentic" | "cs.ma" | "mcp" => Some("agents"),
        "quantization" | "vllm" | "llama.cpp" | "gguf" | "cuda" | "serving" => Some("inference"),
        "retrieval" | "vector-search" | "embeddings" | "cs.ir" => Some("rag"),
        "cs.lg" | "stat.ml" | "deep-learning" | "neural-networks" | "cs.ai" => Some("ml-theory"),
        "cv" | "cs.cv" | "vision" | "diffusion" | "image-generation" => Some("computer-vision"),
        "robot" | "cs.ro" | "embodied-ai" | "autonomous" => Some("robotics"),
        "audio" | "speech" | "tts" | "video" | "cs.sd" => Some("multimodal"),
        "rl" | "rlhf" | "ppo" | "grpo" => Some("reinforcement-learning"),
        "open-weight" | "huggingface" | "open-model" => Some("open-weights"),

        // Math aliases
        "math.nt" | "primes" | "arithmetic-geometry" => Some("number-theory"),
        "math.co" | "graphs" | "graph-theory" | "discrete-math" => Some("combinatorics"),
        "math.lo" | "set-theory" | "model-theory" | "cs.lo" => Some("logic"),
        "lean" | "lean4" | "coq" | "rocq" | "isabelle" | "agda" | "mathlib"
        | "Automated-theorem-proving"
        | "formal-verification" => Some("formal-proofs"),
        "math.ag" | "math.rt" | "math.ra" | "math.ac" | "algebra" | "geometry" | "math.dg" => {
            Some("algebra-geometry")
        }
        "math.ap" | "math.ca" | "math.fa" | "math.ds" | "pde" | "calculus" | "analysis" => {
            Some("analysis-pde")
        }
        "math.pr" | "math.st" | "statistics" | "probability" | "stochastic" => {
            Some("probability-stats")
        }
        "math.gt" | "math.at" | "algebraic-topology" => Some("topology"),
        "math.oc" | "convex-optimization" | "linear-programming" => Some("optimization"),

        // Physics & Quantum aliases
        "quant-ph" | "qubits" | "qiskit" | "quantum-error-correction" | "qec" => {
            Some("quantum-computing")
        }
        "quantum" | "entanglement" | "quantum-mechanics" => Some("quantum-physics"),
        "cond-mat" | "superconductivity" | "solid-state" | "graphene" => Some("condensed-matter"),
        "astro-ph" | "cosmology" | "black-holes" | "jwst" | "gr-qc" => Some("astrophysics"),
        "hep-th" | "hep-ph" | "hep-ex" | "cern" | "string-theory" | "particle-physics" => {
            Some("high-energy")
        }
        "physics.optics" | "photonics" | "lasers" => Some("optics-photonics"),
        "physics.flu-dyn" | "turbulence" | "thermodynamics" => Some("fluid-dynamics"),

        // Systems, Hardware & Security aliases
        "linux" | "kernel" | "os" | "cs.os" | "ebpf" | "unix" => Some("kernel-os"),
        "chips" | "gpu" | "asic" | "fpga" | "risc-v" | "tsmc" | "hardware" => {
            Some("semiconductors")
        }
        "cs.ar" | "cpu" | "microarchitecture" | "memory" => Some("computer-architecture"),
        "security" | "cve" | "exploit" | "zero-day" | "cybersecurity" | "appsec" => {
            Some("infosec")
        }
        "cs.ni" | "tcp" | "quic" | "bgp" | "dns" | "network" => Some("networking"),
        "iot" | "firmware" | "rtos" | "microcontroller" => Some("embedded"),
        "benchmark" | "profiling" | "simd" | "latency" => Some("performance"),

        // Computational Science & Bio aliases
        "q-bio" | "q-bio.qm" | "computational-biology" => Some("bioinformatics"),
        "q-bio.gn" | "q-bio.bm" | "crispr" | "alphafold" | "protein" | "dna" | "rna" => {
            Some("genomics-proteins")
        }
        "q-bio.nc" | "brain" | "fMRI" | "connectome" => Some("neuroscience"),
        "materials" | "battery" | "chemistry" | "polymers" => Some("materials-science"),
        "fusion" | "solar" | "energy" | "nuclear" => Some("clean-tech"),
        "climate" | "physics.ao-ph" | "geophysics" => Some("climate-earth"),

        // Crypto, Web3 & Blockchain aliases
        "cs.cr" | "iacr" | "encryption" | "signatures" | "mpc" | "fhe"
        | "homomorphic-encryption" => Some("cryptography"),
        "zk" | "zero-knowledge" | "zk-snarks" | "zk-starks" | "snark" | "plonk" | "zkvm" => {
            Some("zkp")
        }
        "pqc" | "lattice" | "kyber" | "dilithium" => Some("post-quantum"),
        "bft" | "blockchain" | "ethereum" | "bitcoin" | "l2" | "rollup" => Some("consensus"),
        "solidity" | "evm" | "smart-contract" | "audit" | "vyper" => Some("smart-contracts"),
        "mev" | "defi" | "amm" | "flashbots" => Some("mev-defi"),

        _ => None,
    }
}

/// Normalizes a list of raw tags into a deduplicated, ordered list of canonical tag slugs (max 4).
pub fn normalize_tags(raw_tags: &[String], fallback_area_slug: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();

    for raw in raw_tags {
        if let Some(canon) = resolve_canonical_tag(raw) {
            if seen.insert(canon) {
                out.push(canon.to_string());
                if out.len() >= 4 {
                    break;
                }
            }
        }
    }

    if out.is_empty() {
        // Pick the default canonical tag for this area if none matched
        let default_tag = match fallback_area_slug {
            "cs" => "software-architecture",
            "ai" => "ml-theory",
            "math" => "algebra-geometry",
            "physics" => "quantum-physics",
            "systems" => "kernel-os",
            "bio" => "bioinformatics",
            "crypto" => "cryptography",
            _ => "open-source",
        };
        out.push(default_tag.to_string());
    }

    out
}
