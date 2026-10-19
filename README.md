# Ahbar — Autonomous Daily Technical & Scientific Dispatches

The project was created using AI.

**Ahbar** is an autonomous daily technical and scientific news/preprint aggregator, web archive, and passwordless email digest built with **Rust** (`ahbar-shared`, `ahbar-engine`, `ahbar-api`) and a zero-framework editorial frontend (`index.html`, `style.css`, `app.js`).

Every morning at **`05:00 UTC` (`08:00 TSİ`)**—immediately following arXiv's daily announcement batch—Ahbar ingests 28+ primary academic and engineering feeds, clusters cross-source duplicates via Jaccard title similarity and normalized URLs, scores candidates with a multi-signal + novelty/rigor formula, and emits static JSON issues, per-area/per-tag Atom XML feeds, and inline-styled HTML email digests.

---

## I. Seven Scientific & Technical Areas

| Numeral | Slug | Area | Primary Coverage |
| :--- | :--- | :--- | :--- |
| **I** | `cs` | **Computer Science & Software** | Programming languages, compilers, distributed systems, databases, formal verification, open-source software |
| **II** | `ai` | **AI & Machine Learning** | Large language models, autonomous agents, statistical learning theory, computer vision, robotics, inference systems |
| **III** | `math` | **Mathematics** | Number theory, combinatorics, logic, formal proof assistants (`Lean 4` / `Rocq`), algebra, geometry, analysis |
| **IV** | `physics` | **Physics & Quantum** | Quantum computing & fault tolerance, condensed matter, astrophysics, high-energy theory, optics & photonics |
| **V** | `systems` | **Systems, Hardware & Security** | Operating systems, kernels, semiconductors (`RISC-V`, ASICs), computer architecture, networking, offensive/defensive InfoSec |
| **VI** | `bio` | **Computational Science & Bio** | Bioinformatics, protein design, genomics, computational neuroscience, materials science, climate & fusion tech |
| **VII** | `crypto` | **Cryptography, Web3 & Blockchain** | Zero-knowledge proof systems (`SNARKs`/`STARKs`), post-quantum cryptography, consensus protocols, smart contract security, MEV |

---

## II. Key Architectural Features

1. **Two-Layer Technical Structure:**
   - **Layer 1 (Feed & Email Digest):** A sharp 1–2 sentence technical hook stating *what was proved, built, or measured*, followed by a single dense technical summary paragraph (with native **KaTeX** `$...$` rendering on the web and Unicode math fallback in emails), canonical tags (`#llm`, `#pl-compilers`, `#zk-proofs`, etc.), and direct action badges (`[Analysis & Cite]`, `[Source ↗]`, `[PDF]`, `[HTML / ar5iv]`, `[Code]`, `[HN / Lobsters]`).
   - **Layer 2 (Item Detail Page `?item=<id>`):** Structured 3-part technical breakdown (*I. Problem & Technical Context*, *II. Method, Architecture & Formal Construction*, *III. Empirical Findings, Benchmarks & Trade-offs*), collapsible original paper abstract, one-click **Copy BibTeX** & **Copy Markdown**, clustered cross-source links, and related dispatches.
2. **Weekday vs. Weekend Edition Lifecycle:**
   - **Monday–Friday (`Daily Issue #N`):** Prioritizes fresh arXiv (`cs.*`, `math.*`, `quant-ph`, `cond-mat`, `q-bio`), IACR ePrint, Quanta Magazine, ACM Queue, LWN, and high-signal engineering releases.
   - **Saturday–Sunday (`Weekend Edition #N`):** Automatically switches to deep-dive essays, systems post-mortems, and top weekly breakthroughs when academic preprint servers pause announcements.
3. **Strict Guardrails (`crates/shared/src/guardrails.rs` & `tags.rs`):**
   - Enforces a closed vocabulary of **41 canonical tags** (with automatic alias mapping) so LLM outputs never pollute the taxonomy.
   - Balances LaTeX `$` delimiters for web KaTeX and converts LaTeX macros (`\alpha`, `\le`, `\mathcal{O}`, subscripts/superscripts) into clean Unicode for email clients.
4. **Passwordless Email Subscriptions (`crates/api` & `worker/`):**
   - Zero passwords. Preferences and one-click `List-Unsubscribe-Post` (RFC 8058) links are authenticated with stateless `HMAC-SHA256` tokens.
   - Subscribers customize 4 axes: **Mode** (*Hybrid*, *Standard*, *Personalized*), **Density** (*Compact 5–7*, *Comprehensive 15–20*), **Schedule** (*Daily*, *Weekdays Only*, *Weekend Only*), and **Content Filter** (*All*, *Papers Only*, *Code Only*).

---

## III. Repository Layout

```text
.
├── Cargo.toml                  # Rust workspace manifest
├── crates/
│   ├── shared/                 # Domain models, 41 canonical tags, 28+ sources, LaTeX/Unicode guardrails, HMAC auth
│   ├── engine/                 # Async RSS/Atom/arXiv/HN ingestor, Jaccard deduplicator, LLM synthesizer, JSON/Atom/Email emitter
│   └── api/                    # Self-hosted Axum + SQLite API for passwordless subscriptions & Signal/Noise votes
├── worker/
│   ├── index.js                # Serverless Cloudflare Worker + D1 edge alternative to crates/api
│   ├── schema.sql              # D1 / SQLite schema (subscribers, item_votes)
│   └── wrangler.toml           # Cloudflare Wrangler configuration
├── index.html                  # Zero-framework static web shell
├── style.css                   # Paper & Ink + Functional Red stylesheet (Source Serif 4 + tabular-nums)
├── app.js                      # Client runtime (filtering, ?item=<id> detail view, KaTeX, Saved export, keyboard nav)
├── fonts/                      # Self-hosted Source Serif 4 WOFF2 fonts
├── data/                       # Generated daily issues (latest.json, archive.json, tags.json, health.json, sample-email.html)
└── feed/                       # Generated Atom XML feeds (all.xml, area-*.xml, tag-*.xml)
```

---

## IV. Local Development & Running the Rust Engine

### 1. Run Unit Tests
```bash
cargo test --workspace
```

### 2. Generate Today's Issue & Feeds Locally
```bash
cargo run -p ahbar-engine -- --root .
```
- If `OPENAI_API_KEY` is set in your environment, `ahbar-engine` uses the configured LLM (`OPENAI_MODEL`, default `gpt-4o-mini`) to synthesize Layer-1 hooks and Layer-2 technical breakdowns.
- Without `OPENAI_API_KEY`, the engine runs deterministically using heuristic extraction + curated high-rigor baseline dispatches so local development and CI work out of the box with zero secrets.

### 3. Preview the Web App
Serve the repository root with any static HTTP server:
```bash
python -m http.server 8080
```
Then open `http://localhost:8080`.

### 4. Run the Self-Hosted Subscription API (Optional)
```bash
cargo run -p ahbar-api
```
Listens on `127.0.0.1:8787` (configurable via `AHBAR_BIND`) with SQLite persistence (`AHBAR_DB_URL`).
