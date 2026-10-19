# Ahbar — Autonomous Daily Technical, Scientific & Strategic Dispatches

**Ahbar** ([latifkedi.github.io/ahbar](https://latifkedi.github.io/ahbar/)) is an autonomous daily technical, scientific, economic, geopolitical, and philosophical news/preprint aggregator, web archive, and passwordless email digest built with **Rust** (`ahbar-shared`, `ahbar-engine`, `ahbar-api`) and a zero-framework, zero-CDN editorial frontend (`index.html`, `boot.js`, `style.css`, `app.js`) modeled after [`useful-sites`](https://github.com/latifkedi/useful-sites).

Every morning at **`05:00 UTC` (`08:00 TSİ`)**—immediately following arXiv's daily announcement batch—Ahbar ingests **86 primary academic, engineering, economic, and strategic feeds**, clusters cross-source duplicates via 64-bit SimHash (`≤ 3` bits) and Jaccard title similarity, synthesizes bilingual (`TR` + `EN`) Layer-1 hooks and Layer-2 breakdowns via a multi-provider Free LLM Router (`FreeLlmRouter`), and emits split per-language JSON issues (`latest.tr.json`, `latest.en.json`), zero-JS plain-text directories (`k/index.html`, `k/en/index.html`), Atom XML feeds, and `10 × 3 = 30` dispatch HTML email digests.

---

## I. Ten Scientific, Technical & Strategic Areas (`I` – `X`)

| Numeral | Slug | Area (EN / TR) | Primary Coverage |
| :--- | :--- | :--- | :--- |
| **I** | `cs` | **Computer Science & Software** / *Bilgisayar Bilimi ve Yazılım* | Programming languages, compilers, distributed systems, databases, formal verification, open-source software |
| **II** | `ai` | **AI & Machine Learning** / *Yapay Zekâ ve Makine Öğrenmesi* | Large language models, autonomous agents, statistical learning theory, computer vision, robotics, inference systems |
| **III** | `math` | **Mathematics** / *Matematik* | Number theory, combinatorics, logic, formal proof assistants (`Lean 4` / `Rocq`), algebra, geometry, analysis |
| **IV** | `physics` | **Physics & Quantum** / *Fizik ve Kuantum* | Quantum computing & fault tolerance, condensed matter, astrophysics, high-energy theory, optics & photonics |
| **V** | `systems` | **Systems, Hardware & Security** / *Sistemler, Donanım ve Güvenlik* | Operating systems, kernels, semiconductors (`RISC-V`, ASICs), computer architecture, networking, offensive/defensive InfoSec |
| **VI** | `bio` | **Computational Science & Bio** / *Hesaplamalı Bilim ve Biyoloji* | Bioinformatics, protein design, genomics, computational neuroscience, materials science, climate & fusion tech |
| **VII** | `crypto` | **Cryptography, Web3 & Blockchain** / *Kriptografi, Web3 ve Blokzincir* | Zero-knowledge proof systems (`SNARKs`/`STARKs`), post-quantum cryptography, consensus protocols, smart contract security, MEV |
| **VIII** | `econ` | **Economics, Markets & Quantitative Finance** / *Ekonomi, Piyasalar ve Kantitatif Finans* | Macroeconomics, monetary policy, econometrics, market microstructure, quantitative finance (`NBER`, `BIS`, `VoxEU`, `TCMB`, `TEPAV`) |
| **IX** | `geopolitics` | **International Relations, Geopolitics & Strategy** / *Uluslararası İlişkiler, Jeopolitik ve Strateji* | Grand strategy, defense & deterrence, tech & cyber policy, energy & supply chains (`Foreign Affairs`, `Carnegie`, `CSIS`, `War on the Rocks`, `EDAM`) |
| **X** | `philosophy` | **Philosophy of Science, Decision Theory & Cognition** / *Bilim Felsefesi, Karar Teorisi ve Biliş* | Philosophy of science & physics, epistemology, decision & game theory, ethics of technology, complex systems (`PhilSci-Archive`, `Aeon`, `Daily Nous`, `SFI`) |

---

## II. Key Architectural Features

1. **`useful-sites` Editorial Discipline & 0ms First Paint:**
   - **Zero External CDNs & Strict CSP:** No third-party scripts or stylesheets (`default-src 'self'`). All mathematical formulas (`$...$` and `$$...$$`) are compiled directly into native HTML5 **MathML (`<math class="m">`)** at build time (`latex_to_html_math`) and client time.
   - **Build-Time Pre-Rendered Home (`<div class="home" data-pre="1">`):** `ahbar-engine` inlines `style.css` and pre-renders the Turkish Home page (Hero + Enso watermark + 3-column Top Picks `ol.hpicks` + 2-column Roman-numeral Fihrist `ol.fx` `I`–`X` with dotted leaders + 30 categorized preview dispatches) directly into `index.html`.
   - **Split Language Payloads (`latest.tr.json` / `latest.en.json`):** `boot.js` detects the active language (`TR` or `EN`) in `<head>` and pre-fetches only that language's JSON payload.
2. **All 4 `useful-sites` Craft Utilities:**
   - **Compare Tray (`?cmp=id1,id2`):** Select up to 3 dispatches (`karşılaştır` / `compare`) to inspect their hooks, methods, benchmarks, and metadata side-by-side in `table.cmp`.
   - **Shareable Saved Shelf (`?s=id1,id2,...`):** Stateless shareable URL for bookmarked dispatches.
   - **Wayback Machine Archive Links (`arşiv` / `archive`):** Direct `web.archive.org` preservation links on every card, detail page, static directory entry, and email item.
   - **Plain-Text Static Edition (`k/index.html` & `k/en/index.html`):** Zero-JavaScript archival edition (`body.static`) for terminal browsers (`lynx`, `w3m`) and crawlers.
3. **Multi-Provider Free LLM Router (`FreeLlmRouter`) + Zero-Key Neural Turkish Translator:**
   - Compatible with [`tashfeenahmed/freellmapi`](https://github.com/tashfeenahmed/freellmapi) and supports automatic waterfall failover across `FREELLMAPI_URL` -> `GEMINI_API_KEY` (`gemini-2.0-flash`) -> `GROQ_API_KEY` (`llama-3.3-70b-versatile`) -> `CEREBRAS_API_KEY` -> `OPENROUTER_API_KEY` (`meta-llama/llama-3.3-70b-instruct:free`) -> `MISTRAL_API_KEY` -> `GITHUB_MODELS_TOKEN`.
   - Even with zero API keys configured, `ahbar-engine` automatically translates live English dispatches into natural Turkish while preserving `$math$` expressions intact.
4. **Daily Email Digest (`10 Categories × 3 Dispatches = 30 Dispatches`):**
   - Features a clickable Roman-numeral quick-jump bar (`I` – `X`) beneath the masthead and 3 top-ranked dispatches per category (`data/sample-email-tr.html` and `data/sample-email.html`).

---

## III. Local Development & Running the Rust Engine

### 1. Run Unit Tests
```bash
cargo test --workspace
```

### 2. Generate Today's Issue, Split Payloads, Static Edition & Emails
```bash
cargo run -p ahbar-engine -- --root .
```

### 3. Preview the Web App
```bash
python -m http.server 8080
```
Then open `http://localhost:8080`.
