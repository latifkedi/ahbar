/* Ahbar — Client application.
   Built directly on the useful-sites editorial architecture:
   - 10 Areas (I – X): CS, AI, Math, Physics, Systems, Bio, Crypto, Econ, Geopolitics, Philosophy
   - Split per-language payloads (data/latest.tr.json & data/latest.en.json) pre-fetched via boot.js
   - Zero external CDNs: native MathML (<math class="m">) rendering for inline $...$ and $$...$$
   - Home view (body.calm.at-home): Hero, Enso watermark, .proof telemetry, .tryq chips,
     3-column Top Picks (ol.hpicks), 2-column Fihrist Index (ol.fx with .ld dotted leaders),
     and 3-per-category preview (30 dispatches) with link to full 2-column view (?all=1)
   - Compare Tray (.tray & ?cmp=id1,id2), Shareable Saved Shelf (?s=id1,id2), Wayback Archive links,
     and Plain-Text Static Edition (k/ & k/en/) */

(function () {
  "use strict";

  var AREAS = [
    {
      slug: "cs",
      numeral: "I",
      en: "Computer Science & Software",
      tr: "Bilgisayar Bilimi & Yazılım",
      desc_en: "Languages, compilers, distributed systems, databases, formal verification and open-source software.",
      desc_tr: "Programlama dilleri, derleyiciler, dağıtık sistemler, veritabanları, biçimsel doğrulama ve açık kaynak.",
      tags: ["compilers", "distributed-systems", "databases", "rust", "algorithms", "software-architecture", "formal-proofs", "open-source"]
    },
    {
      slug: "ai",
      numeral: "II",
      en: "AI & Machine Learning",
      tr: "Yapay Zekâ & Makine Öğrenmesi",
      desc_en: "Large language models, autonomous agents, statistical learning theory, computer vision, robotics and inference.",
      desc_tr: "Büyük dil modelleri, otonom ajanlar, öğrenme kuramı, bilgisayarlı görü, robotik ve çıkarım motorları.",
      tags: ["llm", "agents", "ml-theory", "inference", "open-weights", "rag", "computer-vision", "robotics", "reinforcement-learning"]
    },
    {
      slug: "math",
      numeral: "III",
      en: "Mathematics & Formal Sciences",
      tr: "Matematik & Biçimsel Bilimler",
      desc_en: "Number theory, combinatorics, logic, formal proof assistants (Lean 4 / Rocq), algebra and analysis.",
      desc_tr: "Sayılar kuramı, kombinatorik, mantık, biçimsel kanıt asistanları (Lean 4 / Rocq), cebir ve analiz.",
      tags: ["number-theory", "combinatorics", "formal-proofs", "analysis-pde", "algebra-geometry", "probability-stats", "logic"]
    },
    {
      slug: "physics",
      numeral: "IV",
      en: "Physics, Quantum & Space",
      tr: "Fizik, Kuantum & Uzay",
      desc_en: "Quantum computing, condensed matter, astrophysics, high-energy theory and optics.",
      desc_tr: "Kuantum hesaplama, yoğun madde, astrofizik, yüksek enerji kuramı ve optik/fotonik.",
      tags: ["quantum-computing", "quantum-physics", "condensed-matter", "high-energy", "astrophysics", "optics-photonics"]
    },
    {
      slug: "systems",
      numeral: "V",
      en: "Systems, Hardware & Security",
      tr: "Sistem, Donanım & Siber Güvenlik",
      desc_en: "Operating systems, kernels, semiconductors, computer architecture, networking and InfoSec.",
      desc_tr: "İşletim sistemleri, çekirdek, yarı iletkenler, bilgisayar mimarisi, ağ protokolleri ve siber güvenlik.",
      tags: ["kernel-os", "computer-architecture", "semiconductors", "infosec", "networking", "performance"]
    },
    {
      slug: "bio",
      numeral: "VI",
      en: "Biotech, Medicine & Materials",
      tr: "Biyoteknoloji, Tıp & Malzeme Bilimi",
      desc_en: "Bioinformatics, computational neuroscience, genomics, protein design, materials science and clean tech.",
      desc_tr: "Biyoinformatik, hesaplamalı sinirbilim, genomik, protein tasarımı, malzeme bilimi ve temiz enerji.",
      tags: ["bioinformatics", "genomics-proteins", "neuroscience", "materials-science", "clean-tech"]
    },
    {
      slug: "crypto",
      numeral: "VII",
      en: "Cryptography, Web3 & Protocols",
      tr: "Kriptografi, Web3 & Dağıtık Protokoller",
      desc_en: "Zero-knowledge proofs, post-quantum cryptography, consensus protocols, smart contract security and MEV.",
      desc_tr: "Sıfır-bilgi kanıtları (ZKP), kuantum-sonrası kriptografi, mutabakat protokolleri ve akıllı sözleşme güvenliği.",
      tags: ["zkp", "cryptography", "post-quantum", "consensus", "smart-contracts"]
    },
    {
      slug: "econ",
      numeral: "VIII",
      en: "Economics, Markets & Quantitative Finance",
      tr: "Ekonomi, Finans & Makro Analiz",
      desc_en: "Macroeconomics, monetary policy, econometrics, quantitative finance, market design and industrial organization.",
      desc_tr: "Makroekonomi, para politikası, ekonometri, kantitatif finans, piyasa tasarımı ve sanayi iktisadı.",
      tags: ["macroeconomics", "monetary-policy", "quant-finance", "econometrics", "market-design", "trade-industry"]
    },
    {
      slug: "geopolitics",
      numeral: "IX",
      en: "International Relations, Geopolitics & Strategy",
      tr: "Uluslararası İlişkiler, Jeopolitik & Strateji",
      desc_en: "Grand strategy, technology & semiconductor policy, defense analysis, energy security and international law.",
      desc_tr: "Küresel strateji, teknoloji ve çip jeopolitiği, savunma analizi, enerji güvenliği ve uluslararası hukuk.",
      tags: ["geopolitics", "tech-policy", "defense-strategy", "energy-security", "international-law", "cyber-statecraft"]
    },
    {
      slug: "philosophy",
      numeral: "X",
      en: "Philosophy of Science, Decision Theory & Cognition",
      tr: "Bilim Felsefesi, Karar Kuramı & Bilişsel Bilimler",
      desc_en: "Philosophy of science, formal epistemology, decision & game theory, cognitive science and complex systems.",
      desc_tr: "Bilim felsefesi, biçimsel epistemoloji, karar ve oyun kuramı, bilişsel bilimler ve karmaşık sistemler.",
      tags: ["philosophy-of-science", "epistemology", "decision-theory", "cognitive-science", "complex-systems", "ai-ethics"]
    }
  ];

  var T = {
    en: {
      title: "Ahbar",
      hero: function (n, issueNo) {
        return "<em>" + n + "</em> technical, scientific &amp; strategic dispatches in Issue #" + issueNo + ". Each one states what was proved, built, or measured.";
      },
      proof: function (n, date, sources, scanned) {
        return n + " qualified dispatches · Issue " + date + " · " + scanned + " raw papers &amp; feeds triaged from " + sources + " primary sources · updated 05:00 UTC · no trackers";
      },
      tryLbl: "Try:",
      tryQs: ["llm", "rust", "zkp", "quantum", "lean 4", "monetary-policy", "geopolitics", "epistemology"],
      hStart: "Today's Top Picks",
      hStartNote: "3 per category · 30 dispatches in daily email",
      fxHead: "Index",
      fxNote: function (a, t) { return a + " areas, " + t + " canonical tags"; },
      homePreviewHead: "Selected Dispatches by Category",
      homePreviewNote: "top 3 per area · 30 dispatches",
      seeAreaAll: function (cnt) { return "All " + cnt + " in this area →"; },
      seeFullList: function (cnt) { return "See all " + cnt + " dispatches in one list →"; },
      allDispatchesTitle: "All Dispatches",
      allAreas: "← Full Index (Home)",
      sideHead: "Index",
      saved: "Saved",
      emailPrev: "Email (10×3) ↗",
      emailHref: "data/sample-email.html",
      staticHref: "k/en/",
      staticLbl: "Plain-text index (k/)",
      subscribe: "Subscribe",
      rand: "Random",
      langBtn: "TR",
      langTitle: "Türkçeye geç",
      ph: "Search dispatches, #tag, author:X, area:Y, min:8, is:paper... (/)",
      filterBtn: function (n) { return "Filter" + (n ? " (" + n + ")" : ""); },
      hasCode: "Has Code",
      papersOnly: "Papers Only",
      discussed: "Discussed",
      sortLbl: "Sort:",
      sorts: { score: "By importance", community: "By community", source: "By source" },
      count: function (n, t) { return n + " / " + t + " dispatches"; },
      empty: "No dispatches match the active filters.",
      clear: "Clear Filters",
      topPickBadge: "Top Pick",
      paperBadge: "Paper",
      codeBadge: "Code",
      analysis: "Analysis & Cite",
      sourceExt: "Source ↗",
      archiveLbl: "archive",
      cmpBtn: "compare",
      cmpOn: "✓ compare",
      cmpGo: function (n) { return "Compare (" + n + ")"; },
      cmpClear: "Clear",
      cmpTitle: "Side-by-Side Comparison",
      cmpRemove: "Remove",
      cmpRows: {
        area: "Area",
        summary: "Summary",
        problem: "I. Problem & Context",
        method: "II. Method & Architecture",
        findings: "III. Findings & Trade-offs",
        source: "Source & Authors",
        links: "Primary Links",
        tags: "Canonical Tags"
      },
      shareShelfBtn: "Copy shareable link (?s=…)",
      shareShelfCopied: "✓ Shareable URL copied",
      saveTip: "Save to local reading list (s)",
      savedEmpty: "Nothing saved yet. Click ☆ or press s on any dispatch to keep it here for Markdown (.md), BibTeX (.bib), or shareable URL (?s=…) export.",
      backArea: "← All dispatches in this area",
      origTitle: "Original Title",
      authorsLbl: "Authors",
      openSource: "Open Primary Source ↗",
      readPdf: "Read PDF ↗",
      readHtml: "Read HTML (ar5iv) ↗",
      openCode: "Code Repository ↗",
      openArchive: "Wayback Archive ↗",
      copyBib: "Copy BibTeX",
      copyMd: "Copy Markdown",
      copied: "✓ Copied",
      audioBriefBtn: "▶ Audio Brief",
      audioPlaying: "Playing",
      audioPaused: "Paused",
      audioTitle: "Daily Audio Digest",
      citationsLbl: "Citations",
      influentialLbl: "influential",
      institutionsLbl: "Affiliations",
      sec1: "I. Problem & Technical Context",
      sec2: "II. Method, Architecture & Formal Construction",
      sec3: "III. Empirical Findings, Benchmarks & Trade-offs",
      absHead: "Original Paper Abstract / Excerpt",
      clusteredHead: "Clustered Sources & Discussions",
      relatedHead: "Related Dispatches in This Issue",
      footText: function (date, sources) {
        return "Autonomous daily technical, scientific &amp; strategic digest, last built <b>" + date + "</b> at <code>05:00 UTC</code> (08:00 TSİ) across <b>" + sources + "</b> primary feeds. Zero marketing copy; every dispatch states what was proved, built, or measured. <a href=\"data/health.json\" target=\"_blank\" rel=\"noopener\">Source health telemetry ↗</a>";
      },
      expLbl: "Export saved:",
      kbFoot: "<kbd>/</kbd> Search · <kbd>Ctrl K</kbd> Commands · <kbd>j</kbd>/<kbd>k</kbd> Navigate · <kbd>Enter</kbd> Analysis · <kbd>s</kbd> Save · <kbd>r</kbd> Random · <kbd>?</kbd> Shortcuts",
      palPh: "Jump to a dispatch, area, #tag, or command…",
      palNone: "Nothing matches",
      gAct: "Actions",
      gArea: "Areas (I – X)",
      gTag: "Canonical Tags",
      gRec: "Dispatches",
      kbTitle: "Keyboard Shortcuts",
      keys: [
        ["/", "Focus search bar"],
        ["Ctrl K · ⌘ K", "Open command palette"],
        ["j · k", "Next · previous dispatch"],
        ["Enter", "Open Layer-2 technical analysis & BibTeX"],
        ["o", "Open primary source URL in new tab"],
        ["s", "Save / unsave focused dispatch"],
        ["r", "Jump to a random dispatch"],
        ["?", "Show keyboard shortcuts"],
        ["Esc", "Clear filters or close dialog"]
      ]
    },
    tr: {
      title: "Ahbar",
      hero: function (n, issueNo) {
        return "Sayı #" + issueNo + " içinde <em>" + n + "</em> teknik, bilimsel ve stratejik bülten kaydı. Her biri neyin kanıtlandığını, inşa edildiğini veya ölçüldüğünü söyler.";
      },
      proof: function (n, date, sources, scanned) {
        return n + " seçilmiş kayıt · " + date + " · " + sources + " birincil kaynaktan " + scanned + " makale ve besleme tarandı · her gün 08:00 TSİ · izleyici yok";
      },
      tryLbl: "Dene:",
      tryQs: ["llm", "rust", "zkp", "quantum", "lean 4", "monetary-policy", "geopolitics", "epistemology"],
      hStart: "Günün Seçkisi (Top Picks)",
      hStartNote: "her kategoriden 3 haber · e-postada 30 kayıt",
      fxHead: "Fihrist",
      fxNote: function (a, t) { return a + " alan, " + t + " kanonik etiket"; },
      homePreviewHead: "Kategorilere Göre Günün Özeti",
      homePreviewNote: "her alandan en önemli 3 kayıt · toplam 30 kayıt",
      seeAreaAll: function (cnt) { return "Bu alandaki tüm " + cnt + " kaydı gör →"; },
      seeFullList: function (cnt) { return "Tüm kayıtları tek listede gör (" + cnt + ") →"; },
      allDispatchesTitle: "Tüm Kayıtlar",
      allAreas: "← Fihrist (Ana Sayfa)",
      sideHead: "Fihrist",
      saved: "Kaydedilenler",
      emailPrev: "E-Posta (10×3) ↗",
      emailHref: "data/sample-email-tr.html",
      staticHref: "k/",
      staticLbl: "Düz metin dizini (k/)",
      subscribe: "Abone Ol",
      rand: "Rastgele",
      langBtn: "EN",
      langTitle: "Switch to English",
      ph: "Bültende ara, #etiket, yazar:X, alan:Y, min:8, is:paper... (/)",
      filterBtn: function (n) { return "Süz" + (n ? " (" + n + ")" : ""); },
      hasCode: "Açık Kodlu",
      papersOnly: "Sadece Makale",
      discussed: "Tartışılanlar",
      sortLbl: "Sırala:",
      sorts: { score: "Önem puanına göre", community: "Topluluk ilgisine göre", source: "Kaynağa göre" },
      count: function (n, t) { return n + " / " + t + " kayıt"; },
      empty: "Seçili süzgeçlerle eşleşen kayıt bulunamadı.",
      clear: "Filtreleri Temizle",
      topPickBadge: "Seçki",
      paperBadge: "Makale",
      codeBadge: "Kod",
      analysis: "Analiz & Atıf",
      sourceExt: "Kaynak ↗",
      archiveLbl: "arşiv",
      cmpBtn: "karşılaştır",
      cmpOn: "✓ karşılaştır",
      cmpGo: function (n) { return "Karşılaştır (" + n + ")"; },
      cmpClear: "Temizle",
      cmpTitle: "Yan Yana Karşılaştırma",
      cmpRemove: "Kaldır",
      cmpRows: {
        area: "Alan",
        summary: "Özet",
        problem: "I. Problem & Bağlam",
        method: "II. Yöntem & Mimari",
        findings: "III. Bulgular & Ödünleşimler",
        source: "Kaynak & Yazarlar",
        links: "Birincil Bağlantılar",
        tags: "Kanonik Etiketler"
      },
      shareShelfBtn: "Paylaşılabilir bağlantı kopyala (?s=…)",
      shareShelfCopied: "✓ Paylaşım bağlantısı kopyalandı",
      saveTip: "Okuma listesine kaydet (s)",
      savedEmpty: "Henüz kaydedilmiş bir haber yok. Herhangi bir kayıttaki ☆ düğmesiyle veya s tuşuyla buraya ekleyip Markdown (.md), BibTeX (.bib) ya da paylaşılabilir bağlantı (?s=…) olarak dışa aktarabilirsin.",
      backArea: "← Bu alandaki tüm kayıtlar",
      origTitle: "Orijinal Başlık",
      authorsLbl: "Yazarlar",
      openSource: "Birincil Kaynağı Aç ↗",
      readPdf: "PDF Oku ↗",
      readHtml: "HTML Oku (ar5iv) ↗",
      openCode: "Kod Deposu ↗",
      openArchive: "Wayback Arşivi ↗",
      copyBib: "BibTeX Kopyala",
      copyMd: "Markdown Kopyala",
      copied: "✓ Kopyalandı",
      audioBriefBtn: "▶ Sesli Bülten",
      audioPlaying: "Okunuyor",
      audioPaused: "Duraklatıldı",
      audioTitle: "Günün Sesli Seçkisi",
      citationsLbl: "Atıf",
      influentialLbl: "etkili",
      institutionsLbl: "Kurumlar",
      sec1: "I. Problem & Teknik Bağlam",
      sec2: "II. Yöntem, Mimari & Biçimsel Yapı",
      sec3: "III. Deneysel Bulgular, Kıyaslamalar & Ödünleşimler",
      absHead: "Orijinal Makale Özeti (Abstract)",
      clusteredHead: "Kümelenmiş Kaynaklar & Tartışmalar",
      relatedHead: "Bu Sayıdaki İlgili Diğer Kayıtlar",
      footText: function (date, sources) {
        return "Otonom günlük teknik, bilimsel ve stratejik bülten; son derleme <b>" + date + "</b> saat <code>08:00 TSİ</code> (<code>05:00 UTC</code>), <b>" + sources + "</b> birincil kaynak tarandı. Pazarlama dili içermez; her kayıt neyin kanıtlandığını veya inşa edildiğini belirtir. <a href=\"data/health.json\" target=\"_blank\" rel=\"noopener\">Kaynak sağlık telemetrisi ↗</a>";
      },
      expLbl: "Kaydedilenleri dışa aktar:",
      kbFoot: "<kbd>/</kbd> Ara · <kbd>Ctrl K</kbd> Komutlar · <kbd>j</kbd>/<kbd>k</kbd> Gezin · <kbd>Enter</kbd> Analiz · <kbd>s</kbd> Kaydet · <kbd>r</kbd> Rastgele · <kbd>?</kbd> Kısayollar",
      palPh: "Bir kayda, alana, #etikete ya da komuta git…",
      palNone: "Eşleşen sonuç yok",
      gAct: "Komutlar",
      gArea: "Alanlar (I – X)",
      gTag: "Kanonik Etiketler",
      gRec: "Kayıtlar",
      kbTitle: "Klavye Kısayolları",
      keys: [
        ["/", "Arama kutusuna odaklan"],
        ["Ctrl K · ⌘ K", "Komut paletini aç"],
        ["j · k", "Sonraki · önceki kayıt"],
        ["Enter", "Katman-2 teknik analiz ve BibTeX sayfasını aç"],
        ["o", "Birincil kaynağı yeni sekmede aç"],
        ["s", "Seçili kaydı okuma listesine ekle / çıkar"],
        ["r", "Rastgele bir teknik kayda git"],
        ["?", "Klavye kısayolları yardımı"],
        ["Esc", "Süzgeçleri temizle ya da pencereyi kapat"]
      ]
    }
  };

  var STORAGE_KEYS = {
    theme: "ahbar.theme.v2",
    lang: "ahbar.lang.v2",
    saved: "ahbar.saved.v2",
    seen: "ahbar.seen.v2",
    votes: "ahbar.votes.v2",
    prefs: "ahbar.prefs.v2"
  };

  var bootLang = (window.__boot && window.__boot.lang) || loadStr(STORAGE_KEYS.lang, "tr");

  var state = {
    lang: bootLang,
    issue: null,
    issueCache: {},
    archive: [],
    canonicalTags: [],
    area: null,
    tag: null,
    query: "",
    sort: "score",
    savedOnly: false,
    sharedShelfIds: [],
    showAllList: false,
    cmpIds: [],
    comparing: false,
    filters: {
      code: false,
      papers: false,
      discussed: false
    },
    activeItemId: null,
    kbIndex: -1,
    visibleItems: [],
    savedMap: loadJson(STORAGE_KEYS.saved, {}),
    seenMap: loadJson(STORAGE_KEYS.seen, {}),
    voteMap: loadJson(STORAGE_KEYS.votes, {}),
    subPrefs: loadJson(STORAGE_KEYS.prefs, {
      email: "",
      mode: "standard",
      density: "three-per-area",
      frequency: "daily",
      content_filter: "all",
      lang: "tr",
      areas: AREAS.map(function (a) { return a.slug; }),
      tags: []
    })
  };

  function loadStr(key, fallback) {
    try {
      return localStorage.getItem(key) || fallback;
    } catch (_) {
      return fallback;
    }
  }

  function saveStr(key, val) {
    try {
      localStorage.setItem(key, val);
    } catch (_) {}
  }

  function loadJson(key, fallback) {
    try {
      var raw = localStorage.getItem(key);
      return raw ? JSON.parse(raw) : fallback;
    } catch (_) {
      return fallback;
    }
  }

  function saveJson(key, val) {
    try {
      localStorage.setItem(key, JSON.stringify(val));
    } catch (_) {}
  }

  function esc(str) {
    return String(str == null ? "" : str)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
  }

  function safeUrl(raw) {
    var s = String(raw == null ? "" : raw).trim();
    if (!s) return "#";
    try {
      var parsed = new URL(s, window.location.origin);
      if (parsed.protocol === "https:" || parsed.protocol === "http:") {
        return parsed.href;
      }
    } catch (_) {}
    return "#";
  }

  function isLikelyLatexMath(expr) {
    if (!expr || expr.length > 160) return false;
    if (/^\s|\s$/.test(expr)) return false;
    if (expr.indexOf(". ") !== -1 || expr.indexOf(".\n") !== -1) return false;
    if (/^[0-9]/.test(expr) && expr.indexOf("\\") === -1) {
      if (/^[0-9.,]+\s+[A-Za-z\u00C0-\u024F]{2,}/.test(expr)) return false;
      if (!/[\^_=<>+\-*\/]/.test(expr) && expr.length > 6) return false;
    }
    return true;
  }

  // Zero-CDN LaTeX-to-MathML formatter matching Rust's `latex_to_html_math`
  function formatMathHtml(input) {
    var s = String(input == null ? "" : input);
    if (s.indexOf("$") === -1) return esc(s);

    var out = "";
    var i = 0;
    while (i < s.length) {
      if (s.charAt(i) === "\\" && i + 1 < s.length && s.charAt(i + 1) === "$") {
        out += "$";
        i += 2;
        continue;
      }
      if (s.charAt(i) === "$") {
        var isDisplay = i + 1 < s.length && s.charAt(i + 1) === "$";
        var dLen = isDisplay ? 2 : 1;
        var start = i + dLen;
        var end = isDisplay ? s.indexOf("$$", start) : s.indexOf("$", start);
        if (end !== -1) {
          var expr = s.slice(start, end);
          if (isDisplay || isLikelyLatexMath(expr)) {
            out += renderMathExpr(expr, isDisplay);
            i = end + dLen;
            continue;
          }
        }
      }
      out += esc(s.charAt(i));
      i += 1;
    }
    return out;
  }

  function renderMathExpr(expr, isDisplay) {
    var u = expr
      .replace(/\\(?:left|right)(?![a-zA-Z])/g, "")
      .replace(/\\times(?![a-zA-Z])/g, "×")
      .replace(/\\cdot(?![a-zA-Z])/g, "·")
      .replace(/\\leq?(?![a-zA-Z])/g, "≤")
      .replace(/\\geq?(?![a-zA-Z])/g, "≥")
      .replace(/\\neq(?![a-zA-Z])/g, "≠")
      .replace(/\\approx(?![a-zA-Z])/g, "≈")
      .replace(/\\sim(?![a-zA-Z])/g, "∼")
      .replace(/\\ll(?![a-zA-Z])/g, "≪")
      .replace(/\\gg(?![a-zA-Z])/g, "≫")
      .replace(/\\(?:to|rightarrow)(?![a-zA-Z])/g, "→")
      .replace(/\\infty(?![a-zA-Z])/g, "∞")
      .replace(/\\pm(?![a-zA-Z])/g, "±")
      .replace(/\\%/g, "%")
      .replace(/\\mathbb\{F\}/g, "𝔽")
      .replace(/\\mathbb\{R\}/g, "ℝ")
      .replace(/\\mathbb\{Z\}/g, "ℤ")
      .replace(/\\mathbb\{N\}/g, "ℕ")
      .replace(/\\mathbb\{E\}/g, "𝔼")
      .replace(/\\mathcal\{O\}/g, "𝒪")
      .replace(/\\alpha(?![a-zA-Z])/g, "α")
      .replace(/\\beta(?![a-zA-Z])/g, "β")
      .replace(/\\gamma(?![a-zA-Z])/g, "γ")
      .replace(/\\delta(?![a-zA-Z])/g, "δ")
      .replace(/\\Delta(?![a-zA-Z])/g, "Δ")
      .replace(/\\(?:varepsilon|epsilon)(?![a-zA-Z])/g, "ε")
      .replace(/\\theta(?![a-zA-Z])/g, "θ")
      .replace(/\\lambda(?![a-zA-Z])/g, "λ")
      .replace(/\\mu(?![a-zA-Z])/g, "μ")
      .replace(/\\nu(?![a-zA-Z])/g, "ν")
      .replace(/\\pi(?![a-zA-Z])/g, "π")
      .replace(/\\rho(?![a-zA-Z])/g, "ρ")
      .replace(/\\sigma(?![a-zA-Z])/g, "σ")
      .replace(/\\tau(?![a-zA-Z])/g, "τ")
      .replace(/\\omega(?![a-zA-Z])/g, "ω")
      .replace(/\\Omega(?![a-zA-Z])/g, "Ω")
      .replace(/\{,\}/g, ",")
      .replace(/\\text\{([^}]*)\}/g, "$1")
      .replace(/\\([a-zA-Z]+)/g, "$1");

    var body = "";
    var i = 0;
    while (i < u.length) {
      var ch = u.charAt(i);
      if ((ch === "^" || ch === "_") && i + 1 < u.length) {
        var tag = ch === "^" ? "sup" : "sub";
        i += 1;
        var grp = "";
        if (u.charAt(i) === "{") {
          i += 1;
          while (i < u.length && u.charAt(i) !== "}") {
            grp += esc(u.charAt(i));
            i += 1;
          }
          if (i < u.length && u.charAt(i) === "}") i += 1;
        } else {
          while (i < u.length && /[a-zA-Z0-9+\-\/]/.test(u.charAt(i))) {
            grp += esc(u.charAt(i));
            i += 1;
          }
        }
        body += "<" + tag + ">" + grp + "</" + tag + ">";
        continue;
      }
      if (ch === "{" || ch === "}") {
        i += 1;
        continue;
      }
      body += esc(ch);
      i += 1;
    }

    return isDisplay
      ? '<math display="block" class="m m-block"><mrow>' + body + "</mrow></math>"
      : '<math display="inline" class="m"><mrow>' + body + "</mrow></math>";
  }

  function tr() {
    return T[state.lang] || T.tr;
  }

  function getArea(slug) {
    return AREAS.find(function (a) { return a.slug === slug; }) || AREAS[0];
  }

  function areaTitle(areaObj) {
    return state.lang === "tr" ? areaObj.tr : areaObj.en;
  }

  function areaDesc(areaObj) {
    return state.lang === "tr" ? areaObj.desc_tr : areaObj.desc_en;
  }

  function itemHeadline(item) {
    if (state.lang === "tr" && item.headline_tr) return item.headline_tr;
    return item.headline;
  }

  function itemSummary(item) {
    if (state.lang === "tr" && item.summary_tr) return item.summary_tr;
    return item.summary;
  }

  function itemSummarySimple(item) {
    if (state.lang === "tr") {
      if (item.summary_simple_tr) return item.summary_simple_tr;
      if (item.summary_tr) return item.summary_tr;
    }
    if (item.summary_simple) return item.summary_simple;
    return item.summary || "";
  }

  function itemSummaryTechnical(item) {
    if (state.lang === "tr") {
      if (item.summary_technical_tr) return item.summary_technical_tr;
      if (item.detail && item.detail.method_architecture_tr) return item.detail.method_architecture_tr;
    }
    if (item.summary_technical) return item.summary_technical;
    if (item.detail && item.detail.method_architecture) return item.detail.method_architecture;
    return "";
  }

  function itemDetailField(item, fieldName) {
    if (!item.detail) return "";
    if (state.lang === "tr" && item.detail[fieldName + "_tr"]) {
      return item.detail[fieldName + "_tr"];
    }
    return item.detail[fieldName] || "";
  }

  function extractHost(url) {
    try {
      return new URL(url).hostname.replace(/^www\./, "");
    } catch (_) {
      return "";
    }
  }

  // --- Theme Init ---
  function initTheme() {
    var saved = loadStr(STORAGE_KEYS.theme, "");
    if (saved === "dark" || saved === "light") {
      document.documentElement.setAttribute("data-theme", saved);
    }
    var btn = document.getElementById("theme");
    if (btn) {
      btn.onclick = function () {
        var cur = document.documentElement.getAttribute("data-theme");
        var sysDark = window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches;
        var next = (cur || (sysDark ? "dark" : "light")) === "dark" ? "light" : "dark";
        document.documentElement.setAttribute("data-theme", next);
        saveStr(STORAGE_KEYS.theme, next);
      };
    }
  }

  // --- URL State ---
  function readUrlParams() {
    var p = new URLSearchParams(window.location.search);
    var langParam = p.get("lang");
    if (langParam === "tr" || langParam === "en") {
      state.lang = langParam;
      saveStr(STORAGE_KEYS.lang, langParam);
    }
    state.activeItemId = p.get("item") || null;
    var areaParam = p.get("area") || p.get("f");
    state.area = areaParam && AREAS.some(function (a) { return a.slug === areaParam; }) ? areaParam : null;
    state.tag = p.get("tag") || null;
    state.query = p.get("q") || "";
    state.showAllList = p.get("all") === "1";

    var voteParam = p.get("vote");
    if ((voteParam === "signal" || voteParam === "noise") && state.activeItemId) {
      state.voteMap[state.activeItemId] = voteParam;
      saveJson(STORAGE_KEYS.votes, state.voteMap);
    }

    var sParam = p.get("s");
    if (sParam) {
      state.sharedShelfIds = sParam.split(",").map(function (x) { return x.trim(); }).filter(Boolean);
      state.savedOnly = true;
    } else {
      state.sharedShelfIds = [];
      state.savedOnly = p.get("saved") === "1";
    }

    var cmpParam = p.get("cmp");
    if (cmpParam) {
      state.cmpIds = cmpParam.split(",").map(function (x) { return x.trim(); }).filter(Boolean).slice(0, 3);
      state.comparing = state.cmpIds.length > 0;
    } else {
      state.comparing = false;
    }

    var qInput = document.getElementById("q");
    if (qInput) qInput.value = state.query;

    var tok = p.get("token");
    if (tok) state.subPrefs.token = tok;

    if (p.get("prefs") === "1") {
      var em = p.get("email");
      if (em) state.subPrefs.email = em;
      saveJson(STORAGE_KEYS.prefs, state.subPrefs);
      pushUrl(true);
      setTimeout(openSubDialog, 120);
    }
    if (p.get("unsub") === "1") {
      var uem = p.get("email") || state.subPrefs.email || "";
      pushUrl(true);
      setTimeout(function () {
        alert(state.lang === "tr"
          ? uem + " adresi günlük e-posta listesinden çıkarıldı."
          : "One-click unsubscribe completed for " + uem + ".");
      }, 120);
    }
  }

  function pushUrl(replace) {
    var p = new URLSearchParams();
    if (state.lang === "en") p.set("lang", "en");
    if (state.issue && state.archive.length > 0 && state.issue.date !== state.archive[0].date) {
      p.set("issue", state.issue.date);
    }
    if (state.comparing && state.cmpIds.length > 0) {
      p.set("cmp", state.cmpIds.join(","));
    } else if (state.activeItemId) {
      p.set("item", state.activeItemId);
    } else if (state.sharedShelfIds.length > 0) {
      p.set("s", state.sharedShelfIds.join(","));
    } else {
      if (state.showAllList) p.set("all", "1");
      if (state.area) p.set("area", state.area);
      if (state.tag) p.set("tag", state.tag);
      if (state.query) p.set("q", state.query);
      if (state.savedOnly) p.set("saved", "1");
    }
    var qs = p.toString();
    var url = qs ? "?" + qs : window.location.pathname;
    if (replace) window.history.replaceState(null, "", url);
    else window.history.pushState(null, "", url);
  }

  function isAtHome() {
    return (
      !state.activeItemId &&
      !state.comparing &&
      !state.showAllList &&
      !state.area &&
      !state.tag &&
      !state.query.trim() &&
      !state.savedOnly &&
      !state.filters.code &&
      !state.filters.papers &&
      !state.filters.discussed
    );
  }

  // --- Data Loading (Split Language Payloads) ---
  function fetchJson(path) {
    return fetch(path, { cache: "no-cache" }).then(function (r) {
      if (!r.ok) throw new Error("HTTP " + r.status);
      return r.json();
    });
  }

  function fetchIssueForLang(lang, issueDate) {
    var cacheKey = (issueDate || "latest") + ":" + lang;
    if (state.issueCache[cacheKey]) {
      return Promise.resolve(state.issueCache[cacheKey]);
    }
    var primaryPath = issueDate
      ? "data/issues/" + issueDate + "." + lang + ".json"
      : "data/latest." + lang + ".json";
    var fallbackPath = issueDate
      ? "data/issues/" + issueDate + ".json"
      : "data/latest.json";

    return fetchJson(primaryPath)
      .catch(function () {
        return fetchJson(fallbackPath);
      })
      .then(function (iss) {
        if (iss) state.issueCache[cacheKey] = iss;
        return iss;
      });
  }

  function initData() {
    readUrlParams();
    var p = new URLSearchParams(window.location.search);
    var reqIssue = p.get("issue");

    var issuePromise;
    if (!reqIssue && window.__boot && window.__boot.lang === state.lang && window.__boot.issuePromise) {
      issuePromise = window.__boot.issuePromise.then(function (iss) {
        if (iss) {
          state.issueCache["latest:" + state.lang] = iss;
          return iss;
        }
        return fetchIssueForLang(state.lang, null);
      });
    } else {
      issuePromise = fetchIssueForLang(state.lang, reqIssue);
    }

    Promise.allSettled([
      fetchJson("data/archive.json"),
      fetchJson("data/tags.json"),
      issuePromise
    ]).then(function (res) {
      document.documentElement.classList.remove("pending");
      if (res[0].status === "fulfilled" && Array.isArray(res[0].value)) {
        state.archive = res[0].value;
      }
      if (res[1].status === "fulfilled" && Array.isArray(res[1].value)) {
        state.canonicalTags = res[1].value;
      }
      if (res[2].status === "fulfilled" && res[2].value) {
        state.issue = res[2].value;
      }
      renderAll();
    });
  }

  function loadIssueDate(dateStr) {
    fetchIssueForLang(state.lang, dateStr).then(function (iss) {
      state.issue = iss;
      state.activeItemId = null;
      pushUrl(false);
      renderAll();
    });
  }

  function switchLanguage(nextLang) {
    state.lang = nextLang;
    saveStr(STORAGE_KEYS.lang, state.lang);
    var curDate = state.issue && state.archive.length > 0 && state.issue.date !== state.archive[0].date
      ? state.issue.date
      : null;
    fetchIssueForLang(state.lang, curDate).then(function (iss) {
      if (iss) state.issue = iss;
      pushUrl(true);
      renderAll();
    });
  }

  // --- Filtering ---

  // --- Hybrid / Fuzzy / Field-Weighted Search Engine ---
  function foldDiacritics(str) {
    if (!str) return "";
    return str
      .toLowerCase()
      .replace(/[çÇ]/g, "c")
      .replace(/[ğĞ]/g, "g")
      .replace(/[ıİI]/g, "i")
      .replace(/[öÖ]/g, "o")
      .replace(/[şŞ]/g, "s")
      .replace(/[üÜ]/g, "u")
      .replace(/[âÂ]/g, "a")
      .replace(/[îÎ]/g, "i")
      .replace(/[ûÛ]/g, "u")
      .replace(/[éèêë]/g, "e");
  }

  function levenshteinDistance(a, b) {
    if (a === b) return 0;
    var al = a.length, bl = b.length;
    var row = [];
    for (var i = 0; i <= bl; i++) row[i] = i;
    for (var i = 1; i <= al; i++) {
      var prev = i;
      for (var j = 1; j <= bl; j++) {
        var val = (a.charAt(i - 1) === b.charAt(j - 1)) ? row[j - 1] : Math.min(row[j - 1], row[j], prev) + 1;
        row[j - 1] = prev;
        prev = val;
      }
      row[bl] = prev;
    }
    return row[bl];
  }

  function fuzzyMatchText(text, token) {
    var words = text.split(/[\s,.:;!?\"\'()\[\]{}<>\/\\_-]+/);
    for (var i = 0; i < words.length; i++) {
      var w = words[i];
      if (Math.abs(w.length - token.length) <= 1 && w.length >= 4) {
        if (levenshteinDistance(w, token) <= 1) return true;
      }
    }
    return false;
  }

  function parseSearchQuery(rawQuery) {
    var textTokens = [];
    var filters = {
      author: null,
      area: null,
      tag: null,
      minScore: null,
      isPaper: false,
      isCode: false,
      isTop: false
    };

    var parts = rawQuery.trim().split(/\s+/);
    for (var i = 0; i < parts.length; i++) {
      var part = parts[i];
      if (!part) continue;
      var colon = part.indexOf(":");
      if (colon !== -1) {
        var key = part.slice(0, colon).toLowerCase();
        var val = part.slice(colon + 1).toLowerCase();
        if (key === "author" || key === "yazar") {
          filters.author = foldDiacritics(val);
        } else if (key === "area" || key === "alan" || key === "kat") {
          filters.area = val;
        } else if (key === "tag" || key === "etiket") {
          filters.tag = foldDiacritics(val.replace(/^#/, ""));
        } else if (key === "min" || key === "score" || key === "puan") {
          filters.minScore = parseFloat(val) || null;
        } else if (key === "is") {
          if (val === "paper" || val === "makale") filters.isPaper = true;
          if (val === "code" || val === "kod") filters.isCode = true;
          if (val === "top" || val === "seckin") filters.isTop = true;
        } else {
          textTokens.push(foldDiacritics(part));
        }
      } else if (part.charAt(0) === "#") {
        filters.tag = foldDiacritics(part.slice(1));
      } else {
        textTokens.push(foldDiacritics(part));
      }
    }
    return { tokens: textTokens, filters: filters };
  }

  function computeItemSearchScore(item, queryParsed) {
    var f = queryParsed.filters;
    if (f.area && item.area !== f.area) {
      var aMeta = getArea(item.area);
      if (!aMeta || (aMeta.slug !== f.area && foldDiacritics(aMeta.title_tr).indexOf(f.area) === -1)) {
        return 0;
      }
    }
    if (f.tag) {
      var hasTag = (item.tags || []).some(function (t) {
        return foldDiacritics(t).indexOf(f.tag) !== -1;
      });
      if (!hasTag) return 0;
    }
    if (f.author) {
      var hasAuthor = (item.authors || []).some(function (au) {
        return foldDiacritics(au).indexOf(f.author) !== -1;
      });
      if (!hasAuthor) return 0;
    }
    if (f.minScore !== null) {
      if (!item.metrics || item.metrics.final_score < f.minScore) return 0;
    }
    if (f.isPaper && !item.is_paper) return 0;
    if (f.isCode && !(item.links && item.links.code_url)) return 0;
    if (f.isTop && !item.is_top_pick) return 0;

    var tokens = queryParsed.tokens;
    if (tokens.length === 0) return 1.0;

    var headFold = foldDiacritics(itemHeadline(item));
    var origFold = foldDiacritics(item.original_title || "");
    var summaryFold = foldDiacritics(itemSummary(item));
    var tagsFold = (item.tags || []).map(foldDiacritics).join(" ");
    var authorsFold = (item.authors || []).map(foldDiacritics).join(" ");
    var detailFold = foldDiacritics(
      (item.detail && (item.detail.problem_context || "") + " " + (item.detail.method_architecture || "")) || ""
    );

    var totalScore = 0;
    for (var i = 0; i < tokens.length; i++) {
      var tok = tokens[i];
      var tokScore = 0;
      if (headFold.indexOf(tok) !== -1) tokScore += 12;
      if (origFold.indexOf(tok) !== -1) tokScore += 10;
      if (tagsFold.indexOf(tok) !== -1) tokScore += 7;
      if (authorsFold.indexOf(tok) !== -1) tokScore += 5;
      if (summaryFold.indexOf(tok) !== -1) tokScore += 3;
      if (detailFold.indexOf(tok) !== -1) tokScore += 2;

      if (tokScore === 0 && tok.length >= 5) {
        if (fuzzyMatchText(headFold, tok)) tokScore += 6;
        else if (fuzzyMatchText(origFold, tok)) tokScore += 5;
        else if (fuzzyMatchText(tagsFold, tok)) tokScore += 4;
        else if (fuzzyMatchText(summaryFold, tok)) tokScore += 2;
      }

      if (tokScore === 0) {
        return 0;
      }
      totalScore += tokScore;
    }

    return totalScore;
  }

  function matchesItem(item, ignoreArea, ignoreTag) {
    if (state.savedOnly && !state.savedMap[item.id]) return false;
    if (!ignoreArea && state.area && item.area !== state.area) return false;
    if (!ignoreTag && state.tag && (item.tags || []).indexOf(state.tag) === -1) return false;
    if (state.filters.code && !(item.links && item.links.code_url)) return false;
    if (state.filters.papers && !item.is_paper) return false;
    if (state.filters.discussed && !((item.links && item.links.discussion_url) || (item.metrics && item.metrics.community_points > 0))) {
      return false;
    }
    if (state.query.trim()) {
      var qParsed = parseSearchQuery(state.query);
      var score = computeItemSearchScore(item, qParsed);
      if (score <= 0) return false;
      item._matchScore = score;
    }
    return true;
  }

  function getFilteredItems() {
    if (!state.issue || !state.issue.items) return [];
    if (state.sharedShelfIds.length > 0) {
      return state.sharedShelfIds
        .map(function (id) {
          return state.issue.items.find(function (i) { return i.id === id; }) || state.savedMap[id];
        })
        .filter(Boolean);
    }
    if (state.savedOnly) {
      return Object.keys(state.savedMap)
        .map(function (id) {
          return state.issue.items.find(function (i) { return i.id === id; }) || state.savedMap[id];
        })
        .filter(Boolean);
    }
    var list = state.issue.items.filter(function (i) { return matchesItem(i, false, false); });
    list.sort(function (a, b) {
      if (state.query.trim()) {
        var sa = a._matchScore || 0;
        var sb = b._matchScore || 0;
        if (sb !== sa) return sb - sa;
      }
      if (state.sort === "community") {
        var pa = (a.metrics && a.metrics.community_points) || 0;
        var pb = (b.metrics && b.metrics.community_points) || 0;
        if (pb !== pa) return pb - pa;
      } else if (state.sort === "source") {
        var cmp = (a.source_name || "").localeCompare(b.source_name || "");
        if (cmp !== 0) return cmp;
      }
      return ((b.metrics && b.metrics.final_score) || 0) - ((a.metrics && a.metrics.final_score) || 0);
    });
    return list;
  }

  // --- Render Chrome (Header, Sidebar, Compare Tray, Footer) ---
  function renderChrome() {
    var t = tr();
    document.documentElement.lang = state.lang;

    var atHome = isAtHome();
    document.body.classList.toggle("at-home", atHome);
    document.body.classList.toggle("calm", atHome);

    var totalItems = state.issue && state.issue.items ? state.issue.items.length : 90;
    var issueNo = state.issue ? state.issue.issue_number : 1;

    var heroEl = document.getElementById("hero");
    if (heroEl) heroEl.innerHTML = t.hero(totalItems, issueNo);

    var qEl = document.getElementById("q");
    if (qEl) qEl.placeholder = t.ph;

    var audioBtn = document.getElementById("audiobtn");
    if (audioBtn) audioBtn.onclick = toggleAudioBrief;

    var adToggle = document.getElementById("ad-toggle");
    if (adToggle) adToggle.onclick = toggleAudioBrief;

    var adPrev = document.getElementById("ad-prev");
    if (adPrev) adPrev.onclick = prevAudioItem;

    var adNext = document.getElementById("ad-next");
    if (adNext) adNext.onclick = nextAudioItem;

    var adRate = document.getElementById("ad-rate");
    if (adRate) adRate.onclick = cycleAudioRate;

    var adStop = document.getElementById("ad-stop");
    if (adStop) adStop.onclick = stopAudioBrief;

    var audioBtn = document.getElementById("audiobtn");
    if (audioBtn && !audioState.speaking) audioBtn.textContent = t.audioBriefBtn;

    var savedBtn = document.getElementById("savedb");
    var savedCount = Object.keys(state.savedMap).length;
    if (savedBtn) {
      savedBtn.setAttribute("aria-pressed", state.savedOnly ? "true" : "false");
      savedBtn.innerHTML = esc(t.saved) + (savedCount ? ' <span class="c">' + savedCount + "</span>" : "");
    }

    var emailPrev = document.getElementById("emailprev");
    if (emailPrev) {
      emailPrev.textContent = t.emailPrev;
      emailPrev.href = t.emailHref;
    }

    var staticFoot = document.getElementById("static-foot");
    if (staticFoot) {
      staticFoot.textContent = t.staticLbl;
      staticFoot.href = t.staticHref;
    }

    var subBtn = document.getElementById("subbtn");
    if (subBtn) subBtn.textContent = t.subscribe;

    var randBtn = document.getElementById("rand");
    if (randBtn) randBtn.textContent = t.rand;

    var langBtn = document.getElementById("langbtn");
    if (langBtn) {
      langBtn.textContent = t.langBtn;
      langBtn.title = t.langTitle;
    }

    // Compare tray (#tray)
    var trayEl = document.getElementById("tray");
    var trayGo = document.getElementById("tray-go");
    var trayClear = document.getElementById("tray-clear");
    if (trayEl && trayGo && trayClear) {
      if (state.cmpIds.length > 0 && !state.comparing) {
        trayEl.hidden = false;
        trayGo.textContent = t.cmpGo(state.cmpIds.length);
        trayGo.href = "?cmp=" + encodeURIComponent(state.cmpIds.join(","));
        trayClear.textContent = t.cmpClear;
      } else {
        trayEl.hidden = true;
      }
    }

    // Left sticky Fihrist sidebar (#nav)
    var navEl = document.getElementById("nav");
    if (navEl && state.issue) {
      var allItems = state.issue.items || [];
      var activeAreaSlug = state.area;
      if (state.activeItemId) {
        var curItem = allItems.find(function (i) { return i.id === state.activeItemId; });
        if (curItem) activeAreaSlug = curItem.area;
      }

      var navHtml = '<h2 class="side-k">' + esc(t.sideHead) + "</h2><ol>";
      AREAS.forEach(function (a) {
        var cnt = allItems.filter(function (i) { return i.area === a.slug; }).length;
        var isOn = activeAreaSlug === a.slug;
        navHtml +=
          '<li><a href="?area=' + esc(a.slug) + '" class="' + (isOn ? "on" : "") + '" data-area="' + esc(a.slug) + '">' +
          '<span class="rn">' + esc(a.numeral) + "</span>" +
          '<span class="t">' + esc(areaTitle(a)) + "</span>" +
          '<span class="n">' + cnt + "</span>" +
          "</a></li>";
      });
      navHtml += "</ol>";
      navHtml += '<a href="./" class="all" data-home="1">' + esc(t.allAreas) + "</a>";
      navEl.innerHTML = navHtml;
    }

    // Footer
    var footT = document.getElementById("foot-t");
    if (footT && state.issue) {
      var sCount = state.issue.stats ? state.issue.stats.sources_checked : 86;
      footT.innerHTML = t.footText(state.issue.date, sCount);
    }
    var expLbl = document.getElementById("l-exp");
    if (expLbl) expLbl.textContent = t.expLbl;
    var footKb = document.getElementById("foot-kb");
    if (footKb) footKb.innerHTML = t.kbFoot;
  }

  // --- Toolbar (.tb) & Record Item (.rec) HTML Builders ---
  function buildToolbarHtml(shownCount, totalCount) {
    var t = tr();
    var activeFilterCount =
      (state.tag ? 1 : 0) +
      (state.filters.code ? 1 : 0) +
      (state.filters.papers ? 1 : 0) +
      (state.filters.discussed ? 1 : 0);

    var archiveOpts = (state.archive.length ? state.archive : [state.issue]).map(function (entry) {
      var sel = entry.date === state.issue.date ? " selected" : "";
      var lbl = (entry.edition_type === "weekend" ? "Weekend #" : "Issue #") + entry.issue_number + " · " + entry.date;
      return '<option value="' + esc(entry.date) + '"' + sel + ">" + esc(lbl) + "</option>";
    }).join("");

    var activeTagChip = state.tag
      ? '<button type="button" class="chip on" data-clear-tag="1">#' + esc(state.tag) + ' <span class="x">✕</span></button>'
      : "";

    var rssHref = state.tag
      ? "feed/tag-" + encodeURIComponent(state.tag) + ".xml"
      : state.area
        ? "feed/area-" + encodeURIComponent(state.area) + ".xml"
        : "feed/all.xml";

    return (
      '<div class="tb">' +
        '<button type="button" class="chip' + (activeFilterCount ? " on" : "") + '" id="open-filt-btn">' +
          esc(t.filterBtn(activeFilterCount)) +
        "</button>" +
        '<button type="button" class="chip' + (state.filters.code ? " on" : "") + '" data-toggle-filter="code">' +
          esc(t.hasCode) +
        "</button>" +
        '<button type="button" class="chip' + (state.filters.papers ? " on" : "") + '" data-toggle-filter="papers">' +
          esc(t.papersOnly) +
        "</button>" +
        '<button type="button" class="chip' + (state.filters.discussed ? " on" : "") + '" data-toggle-filter="discussed">' +
          esc(t.discussed) +
        "</button>" +
        activeTagChip +
        '<div class="sp">' +
          '<select id="issue-select" aria-label="Issue Date">' + archiveOpts + "</select>" +
          '<label class="sortl"><span>' + esc(t.sortLbl) + '</span>' +
            '<select id="sort">' +
              '<option value="score"' + (state.sort === "score" ? " selected" : "") + ">" + esc(t.sorts.score) + "</option>" +
              '<option value="community"' + (state.sort === "community" ? " selected" : "") + ">" + esc(t.sorts.community) + "</option>" +
              '<option value="source"' + (state.sort === "source" ? " selected" : "") + ">" + esc(t.sorts.source) + "</option>" +
            "</select>" +
          "</label>" +
          '<a href="' + esc(rssHref) + '" target="_blank" rel="noopener" class="arch">Atom ↗</a>' +
          '<span class="count">' + esc(t.count(shownCount, totalCount)) + "</span>" +
        "</div>" +
      "</div>"
    );
  }

  function buildRecHtml(item, seqIndex, showAreaPath) {
    var t = tr();
    var aMeta = getArea(item.area);
    var isSaved = !!state.savedMap[item.id];
    var isSeen = !!state.seenMap[item.id];
    var isCmp = state.cmpIds.indexOf(item.id) !== -1;
    var vote = state.voteMap[item.id] || "";
    var primaryUrl = safeUrl(item.url);
    var host = extractHost(primaryUrl);
    var numStr = String(seqIndex).padStart(2, "0");
    var waybackUrl = "https://web.archive.org/web/*/" + primaryUrl;

    var pathHtml = showAreaPath
      ? '<p class="path"><a href="?area=' + esc(aMeta.slug) + '" data-area="' + esc(aMeta.slug) + '">' +
        esc(aMeta.numeral + " · " + areaTitle(aMeta)) +
        "</a> · " + esc(item.source_name) + "</p>"
      : '<p class="path">' + esc(item.source_name) + (item.authors && item.authors.length ? " · " + esc(item.authors.slice(0, 2).join(", ")) : "") + "</p>";

    var badges = "";
    if (item.is_top_pick) {
      badges += '<span class="pk">' + esc(t.topPickBadge) + "</span>";
    }
    if (item.links && item.links.code_url) {
      badges += '<span class="pk">' + esc(t.codeBadge) + "</span>";
    }

    var tagsHtml = (item.tags || []).map(function (tg) {
      return '<span data-tag="' + esc(tg) + '">#' + esc(tg) + "</span>";
    }).join("<i>·</i>");

    var links = item.links || {};
    var metaLinks = [];
    metaLinks.push('<a href="?item=' + encodeURIComponent(item.id) + '" class="arch on" data-item="' + esc(item.id) + '">' + esc(t.analysis) + "</a>");
    metaLinks.push('<a href="' + esc(primaryUrl) + '" target="_blank" rel="noopener" class="arch">' + esc(t.sourceExt) + "</a>");
    if (links.pdf_url) {
      metaLinks.push('<a href="' + esc(safeUrl(links.pdf_url)) + '" target="_blank" rel="noopener" class="arch">PDF</a>');
    }
    if (links.html_ar5iv_url) {
      metaLinks.push('<a href="' + esc(safeUrl(links.html_ar5iv_url)) + '" target="_blank" rel="noopener" class="arch">ar5iv</a>');
    }
    if (links.code_url) {
      metaLinks.push('<a href="' + esc(safeUrl(links.code_url)) + '" target="_blank" rel="noopener" class="arch">Code</a>');
    }
    metaLinks.push('<a href="' + esc(waybackUrl) + '" target="_blank" rel="noopener" class="arch">' + esc(t.archiveLbl) + "</a>");
    metaLinks.push(
      '<button type="button" class="cmpb" data-cmp="' + esc(item.id) + '" aria-pressed="' + (isCmp ? "true" : "false") + '">' +
      esc(isCmp ? t.cmpOn : t.cmpBtn) + "</button>"
    );
    metaLinks.push(
      '<button type="button" class="arch' + (vote === "signal" ? " on" : "") + '" data-vote="signal" data-item-id="' + esc(item.id) + '">▲ ' +
      item.metrics.final_score.toFixed(1) + "</button>"
    );

    return (
      '<article class="rec' + (isSeen ? " seen" : "") + '" data-item-id="' + esc(item.id) + '">' +
        '<div class="no">' + numStr + "</div>" +
        '<div class="rb">' +
          pathHtml +
          '<div class="nm">' +
            '<a class="name" href="?item=' + encodeURIComponent(item.id) + '" data-item="' + esc(item.id) + '">' +
              formatMathHtml(itemHeadline(item)) +
            "</a>" +
            (host ? '<span class="host">' + esc(host) + "</span>" : "") +
            badges +
            '<button type="button" class="save" data-save="' + esc(item.id) + '" aria-pressed="' + (isSaved ? "true" : "false") + '" title="' + esc(t.saveTip) + '">' +
              (isSaved ? "★" : "☆") +
            "</button>" +
          "</div>" +
          '<p class="desc desc-simple">' +
            '<span class="desc-badge">💡 ' + (state.lang === "tr" ? "Özet" : "Summary") + ':</span> ' +
            formatMathHtml(itemSummarySimple(item)) +
          "</p>" +
          (itemSummaryTechnical(item) && itemSummaryTechnical(item).trim() !== itemSummarySimple(item).trim()
            ? '<details class="desc-tech">' +
                '<summary>🔬 ' + (state.lang === "tr" ? "Teknik Detay &amp; Mimari" : "Technical Mechanism &amp; Architecture") + "</summary>" +
                '<div class="desc-tech-body">' + formatMathHtml(itemSummaryTechnical(item)) + "</div>" +
              "</details>"
            : "") +
          '<div class="mt">' +
            '<span class="itags">' + tagsHtml + "</span>" +
            '<span class="meta">' + metaLinks.join("") + "</span>" +
          "</div>" +
        "</div>" +
      "</article>"
    );
  }

  // --- Main View Renderers ---
  function renderHomeView(mainEl) {
    var t = tr();
    var allItems = state.issue.items || [];
    var stats = state.issue.stats || {};

    // 1. Proof line + Try query chips
    var tryChips = t.tryQs.map(function (q) {
      return '<a class="chip" href="?q=' + encodeURIComponent(q) + '" data-q="' + esc(q) + '">' + esc(q) + "</a>";
    }).join("");

    // 2. Top Picks (6 items in the 3-column ol.hpicks showcase)
    var showcasePicks = [];
    AREAS.forEach(function (a) {
      var topForArea = allItems.find(function (i) { return i.area === a.slug && i.is_top_pick; }) ||
                       allItems.find(function (i) { return i.area === a.slug; });
      if (topForArea && showcasePicks.length < 6) {
        showcasePicks.push(topForArea);
      }
    });

    var hpicksHtml = showcasePicks.map(function (item) {
      var aMeta = getArea(item.area);
      return (
        "<li>" +
          '<a class="hp-title" href="?item=' + encodeURIComponent(item.id) + '" data-item="' + esc(item.id) + '">' +
            formatMathHtml(itemHeadline(item)) +
          "</a>" +
          "<p>" + formatMathHtml(itemSummary(item)) + "</p>" +
          '<div class="n" style="margin-top:6px;"><span class="rn">' + esc(aMeta.numeral) + "</span> · " + esc(areaTitle(aMeta)) + " · " + esc(item.source_name) + "</div>" +
        "</li>"
      );
    }).join("");

    // 3. Fihrist (Index: 10 Areas in 2-column ol.fx with dotted leaders .ld and clickable sub-tags)
    var fxHtml = AREAS.map(function (a) {
      var cnt = allItems.filter(function (i) { return i.area === a.slug; }).length;
      var subTags = a.tags.slice(0, 6).map(function (tg) {
        return '<a href="?area=' + esc(a.slug) + "&tag=" + esc(tg) + '" data-area-tag="' + esc(a.slug + ":" + tg) + '">#' + esc(tg) + "</a>";
      }).join(", ");
      return (
        '<li class="fe">' +
          '<a class="ft" href="?area=' + esc(a.slug) + '" data-area="' + esc(a.slug) + '">' +
            '<span class="rn">' + esc(a.numeral) + "</span>" +
            '<span class="fn">' + esc(areaTitle(a)) + "</span>" +
            '<span class="ld"></span>' +
            '<span class="n">' + cnt + "</span>" +
          "</a>" +
          '<p class="fc">' + subTags + "</p>" +
        "</li>"
      );
    }).join("");

    // 4. Categorized Preview on Home: show ONLY top 3 dispatches per category (30 total)
    var globalSeq = 1;
    var categorizedPreviewHtml = "";
    state.visibleItems = [];
    state.kbIndex = -1;

    AREAS.forEach(function (a) {
      var areaItems = allItems.filter(function (i) { return i.area === a.slug; });
      if (!areaItems.length) return;
      var previewItems = areaItems.slice(0, 3);
      categorizedPreviewHtml +=
        '<section id="sec-' + esc(a.slug) + '">' +
          '<h3 class="sh"><span class="rn">' + esc(a.numeral) + "</span> " +
            '<a href="?area=' + esc(a.slug) + '" data-area="' + esc(a.slug) + '" style="text-decoration:none;">' + esc(areaTitle(a)) + "</a>" +
            ' <span class="n">' + previewItems.length + " / " + areaItems.length + "</span>" +
            '<a class="more-area" href="?area=' + esc(a.slug) + '" data-area="' + esc(a.slug) + '">' + esc(t.seeAreaAll(areaItems.length)) + "</a>" +
          "</h3>" +
          '<div class="recs">' +
            previewItems.map(function (item) {
              state.visibleItems.push(item);
              return buildRecHtml(item, globalSeq++, false);
            }).join("") +
          "</div>" +
        "</section>";
    });

    var canonCount = state.canonicalTags.length || 74;

    // 2.5 Trending Repos & AI Models Showcase
    var trendingItems = allItems.filter(function (i) {
      return (i.tags || []).some(function (tg) {
        return tg === "trending-repo" || tg === "ai-models" || tg === "open-weights" || tg === "open-source" || tg === "consumer-tech" || tg === "hardware";
      }) || (i.links && i.links.code_url);
    }).slice(0, 3);

    var trendingSectionHtml = "";
    if (trendingItems.length > 0) {
      var trendHeading = state.lang === "tr"
        ? "🔥 Haftanın Trend Repoları & Yeni AI Modelleri"
        : "🔥 Trending Repositories & AI Model Releases";
      var trendSub = state.lang === "tr"
        ? "açık kaynak kod, popüler araçlar ve halka yansıyan yeni modeller"
        : "open-source code, trending developer tools & open weights";

      var trendCards = trendingItems.map(function (item) {
        var aMeta = getArea(item.area);
        var sSimple = itemSummarySimple(item);
        return (
          "<li>" +
            '<a class="hp-title" href="?item=' + encodeURIComponent(item.id) + '" data-item="' + esc(item.id) + '">' +
              formatMathHtml(itemHeadline(item)) +
            "</a>" +
            "<p>" + formatMathHtml(sSimple) + "</p>" +
            '<div class="n" style="margin-top:6px;"><span class="rn">' + esc(aMeta.numeral) + "</span> · " + esc(areaTitle(aMeta)) + (item.links && item.links.code_url ? ' · <a href="' + esc(safeUrl(item.links.code_url)) + '" target="_blank" rel="noopener" style="color:var(--red);text-decoration:underline;">GitHub ↗</a>' : '') + "</div>" +
          "</li>"
        );
      }).join("");

      trendingSectionHtml =
        '<section class="hsec" style="margin-top:34px;">' +
          '<h2 class="k">' + esc(trendHeading) + " <span>" + esc(trendSub) + "</span></h2>" +
          '<ol class="trending-grid">' + trendCards + "</ol>" +
        "</section>";
    }

    mainEl.innerHTML =
      '<div class="home">' +
        '<p class="proof">' + t.proof(allItems.length, state.issue.date, stats.sources_checked || 86, stats.raw_items_scanned || 240) + "</p>" +
        '<p class="tryq"><span>' + esc(t.tryLbl) + "</span>" + tryChips + "</p>" +
        '<section class="hsec hp">' +
          '<h2 class="k">' + esc(t.hStart) + "<span>" + esc(t.hStartNote) + "</span></h2>" +
          '<ol class="hpicks">' + hpicksHtml + "</ol>" +
        "</section>" +
        trendingSectionHtml +
        '<section class="hsec">' +
          '<h2 class="k">' + esc(t.fxHead) + "<span>" + esc(t.fxNote(AREAS.length, canonCount)) + "</span></h2>" +
          '<ol class="fx">' + fxHtml + "</ol>" +
          '<p class="hlinks">' +
            '<a href="?all=1" data-all="1">' + esc(t.seeFullList(allItems.length)) + "</a>" +
            '<a href="' + esc(t.emailHref) + '" target="_blank" rel="noopener">' + esc(t.emailPrev) + "</a>" +
            '<a href="' + esc(t.staticHref) + '">' + esc(t.staticLbl) + "</a>" +
          "</p>" +
        "</section>" +
        '<section class="hsec">' +
          '<h2 class="k">' + esc(t.homePreviewHead) + "<span>" + esc(t.homePreviewNote) + "</span></h2>" +
          categorizedPreviewHtml +
          '<p class="hlinks" style="margin-top:28px;">' +
            '<a class="btn" href="?all=1" data-all="1">' + esc(t.seeFullList(allItems.length)) + "</a>" +
          "</p>" +
        "</section>" +
      "</div>";
  }

  function renderCompareView(mainEl) {
    var t = tr();
    var allItems = state.issue.items || [];
    var cmpItems = state.cmpIds
      .map(function (id) {
        return allItems.find(function (i) { return i.id === id; }) || state.savedMap[id];
      })
      .filter(Boolean);

    if (cmpItems.length === 0) {
      state.comparing = false;
      renderAll();
      return;
    }

    var headCols = cmpItems.map(function (item) {
      var host = extractHost(item.url);
      return (
        "<th>" +
          '<a href="?item=' + encodeURIComponent(item.id) + '" data-item="' + esc(item.id) + '">' +
            formatMathHtml(itemHeadline(item)) +
          "</a>" +
          '<span class="host">' + esc(item.source_name) + (host ? " · " + esc(host) : "") + "</span>" +
          '<button type="button" class="lnk" data-cmp-remove="' + esc(item.id) + '">✕ ' + esc(t.cmpRemove) + "</button>" +
        "</th>"
      );
    }).join("");

    function rowHtml(label, cellFn) {
      return (
        "<tr><th>" + esc(label) + "</th>" +
        cmpItems.map(function (item) { return "<td>" + cellFn(item) + "</td>"; }).join("") +
        "</tr>"
      );
    }

    var rows = [
      rowHtml(t.cmpRows.area, function (item) {
        var aMeta = getArea(item.area);
        return '<span class="rn">' + esc(aMeta.numeral) + "</span> " + esc(areaTitle(aMeta));
      }),
      rowHtml(t.cmpRows.summary, function (item) {
        return formatMathHtml(itemSummary(item));
      }),
      rowHtml(t.cmpRows.problem, function (item) {
        return formatMathHtml(itemDetailField(item, "problem_context"));
      }),
      rowHtml(t.cmpRows.method, function (item) {
        return formatMathHtml(itemDetailField(item, "method_architecture"));
      }),
      rowHtml(t.cmpRows.findings, function (item) {
        return formatMathHtml(itemDetailField(item, "findings_tradeoffs"));
      }),
      rowHtml(t.cmpRows.source, function (item) {
        return esc(item.original_title) + (item.authors && item.authors.length ? "<br><span class=\"n\">" + esc(item.authors.join(", ")) + "</span>" : "");
      }),
      rowHtml(t.cmpRows.links, function (item) {
        var links = item.links || {};
        var primaryUrl = safeUrl(item.url);
        var parts = ['<a class="arch" href="' + esc(primaryUrl) + '" target="_blank" rel="noopener">' + esc(t.sourceExt) + "</a>"];
        if (links.pdf_url) parts.push('<a class="arch" href="' + esc(safeUrl(links.pdf_url)) + '" target="_blank" rel="noopener">PDF</a>');
        if (links.code_url) parts.push('<a class="arch" href="' + esc(safeUrl(links.code_url)) + '" target="_blank" rel="noopener">Code</a>');
        parts.push('<a class="arch" href="https://web.archive.org/web/*/' + esc(primaryUrl) + '" target="_blank" rel="noopener">' + esc(t.archiveLbl) + "</a>");
        return parts.join(" · ");
      }),
      rowHtml(t.cmpRows.tags, function (item) {
        return (item.tags || []).map(function (tg) { return "#" + esc(tg); }).join(" · ");
      })
    ].join("");

    mainEl.innerHTML =
      '<div class="fieldpage">' +
        '<p class="crumb"><a href="./" data-home="1">' + esc(t.fxHead) + "</a><b>›</b>" + esc(t.cmpTitle) + "</p>" +
        '<h1 class="ph">' + esc(t.cmpTitle) + ' <span class="n">' + cmpItems.length + "</span></h1>" +
        '<div class="cmpwrap"><table class="cmp">' +
          "<thead><tr><th></th>" + headCols + "</tr></thead>" +
          "<tbody>" + rows + "</tbody>" +
        "</table></div>" +
      "</div>";
  }

  function renderFilteredView(mainEl) {
    var t = tr();
    var allItems = state.issue.items || [];
    var filtered = getFilteredItems();
    state.visibleItems = filtered;
    state.kbIndex = -1;

    var headerHtml = "";
    if (state.savedOnly) {
      var shelfIds = filtered.map(function (i) { return i.id; });
      var shelfBar = shelfIds.length > 0
        ? '<div class="shelfx"><button type="button" class="btn" id="copy-shelf-btn">' + esc(t.shareShelfBtn) + "</button></div>"
        : "";
      headerHtml =
        '<p class="crumb"><a href="./" data-home="1">' + esc(t.fxHead) + "</a><b>›</b>" + esc(t.saved) + "</p>" +
        '<h1 class="ph">' + esc(t.saved) + ' <span class="n">' + filtered.length + "</span></h1>" +
        shelfBar;
    } else if (state.area) {
      var aMeta = getArea(state.area);
      headerHtml =
        '<p class="crumb"><a href="./" data-home="1">' + esc(t.fxHead) + "</a><b>›</b>" + esc(aMeta.numeral + " · " + areaTitle(aMeta)) +
        (state.tag ? "<b>›</b>#" + esc(state.tag) : "") + "</p>" +
        '<h1 class="ph"><span class="rn">' + esc(aMeta.numeral) + "</span> " + esc(areaTitle(aMeta)) + "</h1>" +
        '<p class="lede">' + esc(areaDesc(aMeta)) + "</p>";
    } else if (state.tag) {
      headerHtml =
        '<p class="crumb"><a href="./" data-home="1">' + esc(t.fxHead) + "</a><b>›</b>#" + esc(state.tag) + "</p>" +
        '<h1 class="ph">#' + esc(state.tag) + "</h1>";
    } else if (state.query) {
      headerHtml =
        '<p class="crumb"><a href="./" data-home="1">' + esc(t.fxHead) + "</a><b>›</b>“" + esc(state.query) + "”</p>" +
        '<h1 class="ph">“' + esc(state.query) + '” <span class="n">' + filtered.length + "</span></h1>";
    } else {
      headerHtml =
        '<p class="crumb"><a href="./" data-home="1">' + esc(t.fxHead) + "</a><b>›</b>" + esc(t.allDispatchesTitle) + "</p>" +
        '<h1 class="ph">' + esc(t.allDispatchesTitle) + ' <span class="n">' + filtered.length + "</span></h1>";
    }

    var listHtml = "";
    if (filtered.length === 0) {
      listHtml =
        '<div class="none">' +
          "<p>" + esc(state.savedOnly ? t.savedEmpty : t.empty) + "</p>" +
          (!state.savedOnly ? '<p><a href="./" data-home="1">' + esc(t.clear) + "</a></p>" : "") +
        "</div>";
    } else {
      listHtml =
        '<div class="recs">' +
          filtered.map(function (item, idx) {
            return buildRecHtml(item, idx + 1, !state.area);
          }).join("") +
        "</div>";
    }

    mainEl.innerHTML =
      '<div class="fieldpage">' +
        headerHtml +
        buildToolbarHtml(filtered.length, allItems.length) +
        listHtml +
      "</div>";

    var copyShelfBtn = document.getElementById("copy-shelf-btn");
    if (copyShelfBtn) {
      copyShelfBtn.onclick = function () {
        var ids = filtered.map(function (i) { return i.id; }).join(",");
        var shareUrl = window.location.origin + window.location.pathname + "?s=" + encodeURIComponent(ids);
        navigator.clipboard.writeText(shareUrl).then(function () {
          copyShelfBtn.textContent = t.shareShelfCopied;
          setTimeout(function () { copyShelfBtn.textContent = t.shareShelfBtn; }, 1800);
        });
      };
    }
  }

  function buildCoinsTitle(item) {
    var params = [
      "ctx_ver=Z39.88-2004",
      "rft_val_fmt=info%3Aofi%2Ffmt%3Akev%3Amtx%3Ajournal",
      "rft.genre=article",
      "rft.atitle=" + encodeURIComponent(item.original_title || item.headline || ""),
      "rft.jtitle=" + encodeURIComponent(item.source_name || "Ahbar"),
      "rft.date=" + encodeURIComponent(item.issue_date || ""),
      "rft_id=" + encodeURIComponent(item.url || "")
    ];
    if (item.authors && item.authors.length) {
      item.authors.forEach(function (au) {
        params.push("rft.au=" + encodeURIComponent(au));
      });
    }
    return params.join("&");
  }

  function updateAcademicMeta(item) {
    clearAcademicMeta();
    var head = document.head;
    if (!head) return;
    function addMeta(name, val) {
      if (!val) return;
      var m = document.createElement("meta");
      m.setAttribute("name", name);
      m.setAttribute("content", val);
      m.setAttribute("data-academic", "1");
      head.appendChild(m);
    }
    addMeta("citation_title", item.original_title || item.headline);
    addMeta("citation_publication_date", item.issue_date || "");
    addMeta("citation_journal_title", "Ahbar — " + (item.source_name || "Scientific Dispatch"));
    addMeta("citation_public_url", item.url);
    if (item.authors && item.authors.length) {
      item.authors.forEach(function (au) {
        addMeta("citation_author", au);
      });
    }
    var links = item.links || {};
    if (links.pdf_url) {
      addMeta("citation_pdf_url", links.pdf_url);
    }
    if (item.url && item.url.indexOf("arxiv.org") !== -1) {
      var arx = item.url.replace(/^.*\/abs\//, "").replace(/^.*\/pdf\//, "").replace(/\.pdf$/, "");
      if (arx) addMeta("citation_arxiv_id", arx);
    }
  }

  function clearAcademicMeta() {
    var existing = document.querySelectorAll('meta[data-academic="1"]');
    for (var i = 0; i < existing.length; i++) {
      existing[i].parentNode.removeChild(existing[i]);
    }
  }

  function renderDetailView(mainEl, item) {
    var t = tr();
    state.seenMap[item.id] = Date.now();
    saveJson(STORAGE_KEYS.seen, state.seenMap);
    updateAcademicMeta(item);

    var aMeta = getArea(item.area);
    var isSaved = !!state.savedMap[item.id];
    var links = item.links || {};
    var detail = item.detail || {};
    var primaryUrl = safeUrl(item.url);
    var host = extractHost(primaryUrl);
    var waybackUrl = "https://web.archive.org/web/*/" + primaryUrl;

    var visitBtns = [];
    visitBtns.push('<a class="btn primary" href="' + esc(primaryUrl) + '" target="_blank" rel="noopener">' + esc(t.openSource) + "</a>");
    if (links.pdf_url) {
      visitBtns.push('<a class="btn" href="' + esc(safeUrl(links.pdf_url)) + '" target="_blank" rel="noopener">' + esc(t.readPdf) + "</a>");
    }
    if (links.html_ar5iv_url) {
      visitBtns.push('<a class="btn" href="' + esc(safeUrl(links.html_ar5iv_url)) + '" target="_blank" rel="noopener">' + esc(t.readHtml) + "</a>");
    }
    if (links.code_url) {
      visitBtns.push('<a class="btn" href="' + esc(safeUrl(links.code_url)) + '" target="_blank" rel="noopener">' + esc(t.openCode) + "</a>");
    }
    visitBtns.push('<a class="btn" href="' + esc(waybackUrl) + '" target="_blank" rel="noopener">' + esc(t.openArchive) + "</a>");
    if (detail.bibtex) {
      visitBtns.push('<button type="button" class="btn" id="copy-bib-btn">' + esc(t.copyBib) + "</button>");
    }
    visitBtns.push('<button type="button" class="btn" id="copy-md-btn">' + esc(t.copyMd) + "</button>");

    var tagsHtml = (item.tags || []).map(function (tg) {
      return '<span data-tag="' + esc(tg) + '">#' + esc(tg) + "</span>";
    }).join("<i>·</i>");

    var clusteredHtml = "";
    if (links.clustered_sources && links.clustered_sources.length > 0) {
      clusteredHtml =
        '<h2 class="k" style="margin-top:28px;">' + esc(t.clusteredHead) + "</h2>" +
        '<ul class="vs">' +
          links.clustered_sources.map(function (cs) {
            return (
              "<li>" +
                '<a href="' + esc(safeUrl(cs.url)) + '" target="_blank" rel="noopener">' + esc(cs.source_name) + " ↗</a>" +
                '<span class="ax">' + esc(cs.title) + (cs.points ? " (" + cs.points + " pts)" : "") + "</span>" +
              "</li>"
            );
          }).join("") +
        "</ul>";
    }

    var related = (state.issue.items || []).filter(function (o) {
      return o.id !== item.id && (o.area === item.area || (o.tags || []).some(function (tg) { return (item.tags || []).indexOf(tg) !== -1; }));
    }).slice(0, 4);

    var relatedHtml = "";
    if (related.length > 0) {
      relatedHtml =
        '<h2 class="k" style="margin-top:28px;">' + esc(t.relatedHead) + "</h2>" +
        '<ul class="vs">' +
          related.map(function (r) {
            return (
              "<li>" +
                '<a href="?item=' + encodeURIComponent(r.id) + '" data-item="' + esc(r.id) + '">' + formatMathHtml(itemHeadline(r)) + "</a>" +
                '<span class="ax">' + esc(r.source_name) + "</span>" +
              "</li>"
            );
          }).join("") +
        "</ul>";
    }

    mainEl.innerHTML =
      '<div class="one">' +
        '<p class="crumb">' +
          '<a href="./" data-home="1">' + esc(t.fxHead) + "</a><b>›</b>" +
          '<a href="?area=' + esc(aMeta.slug) + '" data-area="' + esc(aMeta.slug) + '">' + esc(aMeta.numeral + " · " + areaTitle(aMeta)) + "</a><b>›</b>" +
          esc(item.source_name) +
        "</p>" +
        '<article class="rec big">' +
          '<span class="Z3988" title="' + esc(buildCoinsTitle(item)) + '"></span>' +
          '<div class="rb">' +
            '<div class="nm">' +
              '<h1 class="ph"><a class="name" href="' + esc(primaryUrl) + '" target="_blank" rel="noopener">' + formatMathHtml(itemHeadline(item)) + "</a></h1>" +
              '<button type="button" class="save" data-save="' + esc(item.id) + '" aria-pressed="' + (isSaved ? "true" : "false") + '" title="' + esc(t.saveTip) + '">' +
                (isSaved ? "★" : "☆") +
              "</button>" +
            "</div>" +
            '<p class="path" style="margin-top:4px;">' +
              "<b>" + esc(t.origTitle) + ":</b> " + formatMathHtml(item.original_title) +
              (host ? ' · <span class="host">' + esc(host) + "</span>" : "") +
              (item.authors && item.authors.length ? "<br><b>" + esc(t.authorsLbl) + ":</b> " + esc(item.authors.join(", ")) : "") +
            "</p>" +
            '<div class="dual-summary-box" style="margin:16px 0 22px;padding:14px 18px;border-left:3px solid var(--red);background:var(--panel);border-radius:4px;">' +
              '<p style="margin:0 0 10px;font-size:15.5px;line-height:1.6;color:var(--fg);">' +
                '<strong style="color:var(--red);font-size:12.5px;text-transform:uppercase;letter-spacing:.04em;display:block;margin-bottom:4px;">💡 ' + (state.lang === "tr" ? "Sade Özet (Ne Anlama Geliyor?)" : "Layman Summary (What It Means)") + '</strong>' +
                formatMathHtml(itemSummarySimple(item)) +
              '</p>' +
              (itemSummaryTechnical(item) && itemSummaryTechnical(item).trim() !== itemSummarySimple(item).trim()
                ? '<p style="margin:10px 0 0;padding-top:10px;border-top:1px dashed var(--rule2);font-size:14.5px;line-height:1.6;color:var(--dim);">' +
                    '<strong style="color:var(--red);font-size:12.5px;text-transform:uppercase;letter-spacing:.04em;display:block;margin-bottom:4px;">🔬 ' + (state.lang === "tr" ? "Teknik Mekanizma & Mimari" : "Technical Mechanism & Architecture") + '</strong>' +
                    formatMathHtml(itemSummaryTechnical(item)) +
                  '</p>'
                : '') +
            '</div>' +
            '<div class="visit">' + visitBtns.join("") + "</div>" +
            '<div class="mt" style="margin-bottom:28px;">' +
              '<span class="itags">' + tagsHtml + "</span>" +
              '<span class="meta"><span class="n">Score ' + item.metrics.final_score.toFixed(1) + " · Novelty " + item.metrics.llm_novelty.toFixed(1) + " · Rigor " + item.metrics.llm_rigor.toFixed(1) + "</span>" +
              (item.metrics.citation_count ? '<span class="badge-cite">📚 ' + esc(t.citationsLbl) + ': ' + item.metrics.citation_count + (item.metrics.influential_citations ? ' (' + item.metrics.influential_citations + ' ' + esc(t.influentialLbl) + ')' : '') + '</span>' : '') +
              "</span>" +
              (detail.author_institutions && detail.author_institutions.length ? '<span class="badge-inst">🏛️ <b>' + esc(t.institutionsLbl) + ':</b> ' + esc(detail.author_institutions.join(", ")) + '</span>' : '') +
            "</div>" +

            '<h2 class="k">' + esc(t.sec1) + "</h2>" +
            '<p style="margin:10px 0 24px;max-width:70ch;font-size:15.5px;line-height:1.66;color:var(--fg);">' +
              formatMathHtml(itemDetailField(item, "problem_context")) +
            "</p>" +

            '<h2 class="k">' + esc(t.sec2) + "</h2>" +
            '<p style="margin:10px 0 24px;max-width:70ch;font-size:15.5px;line-height:1.66;color:var(--fg);">' +
              formatMathHtml(itemDetailField(item, "method_architecture")) +
            "</p>" +

            '<h2 class="k">' + esc(t.sec3) + "</h2>" +
            '<p style="margin:10px 0 24px;max-width:70ch;font-size:15.5px;line-height:1.66;color:var(--fg);">' +
              formatMathHtml(itemDetailField(item, "findings_tradeoffs")) +
            "</p>" +

            (detail.abstract_excerpt
              ? '<details style="margin:18px 0 24px;padding:12px 0;border-top:1px solid var(--rule2);border-bottom:1px solid var(--rule2);">' +
                  '<summary style="cursor:pointer;font-weight:600;color:var(--dim);">' + esc(t.absHead) + "</summary>" +
                  '<p style="margin:10px 0 0;max-width:70ch;font-size:14.5px;line-height:1.62;color:var(--dim);">' + formatMathHtml(detail.abstract_excerpt) + "</p>" +
                "</details>"
              : "") +

            (detail.bibtex
              ? '<h2 class="k">BibTeX</h2><pre class="bibtex-pre">' + esc(detail.bibtex) + "</pre>"
              : "") +

            clusteredHtml +
            relatedHtml +
            '<p class="onemore" style="margin-top:28px;"><a href="?area=' + esc(aMeta.slug) + '" data-area="' + esc(aMeta.slug) + '">' + esc(t.backArea) + "</a></p>" +
          "</div>" +
        "</article>" +
      "</div>";

    var copyBibBtn = document.getElementById("copy-bib-btn");
    if (copyBibBtn && detail.bibtex) {
      copyBibBtn.onclick = function () {
        navigator.clipboard.writeText(detail.bibtex).then(function () {
          copyBibBtn.textContent = t.copied;
          setTimeout(function () { copyBibBtn.textContent = t.copyBib; }, 1600);
        });
      };
    }

    var copyMdBtn = document.getElementById("copy-md-btn");
    if (copyMdBtn) {
      copyMdBtn.onclick = function () {
        navigator.clipboard.writeText(formatMarkdown(item)).then(function () {
          copyMdBtn.textContent = t.copied;
          setTimeout(function () { copyMdBtn.textContent = t.copyMd; }, 1600);
        });
      };
    }
  }

  function renderAll() {
    if (!state.issue) return;
    renderChrome();
    var mainEl = document.getElementById("list");
    if (!mainEl) return;

    if (state.comparing && state.cmpIds.length > 0) {
      renderCompareView(mainEl);
      return;
    }

    if (state.activeItemId) {
      var found = (state.issue.items || []).find(function (i) { return i.id === state.activeItemId; }) ||
                  state.savedMap[state.activeItemId];
      if (found) {
        renderDetailView(mainEl, found);
        bindToolbarEvents();
        return;
      } else {
        state.activeItemId = null;
      }
    }

    clearAcademicMeta();

    if (isAtHome()) {
      renderHomeView(mainEl);
    } else {
      renderFilteredView(mainEl);
    }

    bindToolbarEvents();
  }

  function bindToolbarEvents() {
    var issSel = document.getElementById("issue-select");
    if (issSel) {
      issSel.onchange = function () {
        loadIssueDate(issSel.value);
      };
    }
    var sortSel = document.getElementById("sort");
    if (sortSel) {
      sortSel.onchange = function () {
        state.sort = sortSel.value;
        renderAll();
      };
    }
    var openFiltBtn = document.getElementById("open-filt-btn");
    if (openFiltBtn) {
      openFiltBtn.onclick = openFilterDialog;
    }
  }

  // --- Dialogs: Filter, Subscribe, Command Palette, Keyboard Help ---
  function openFilterDialog() {
    var dlg = document.getElementById("filt");
    var body = document.getElementById("f-body");
    if (!dlg || !body || !state.issue) return;

    var allItems = state.issue.items || [];
    var tagCounts = {};
    allItems.forEach(function (i) {
      if (!state.area || i.area === state.area) {
        (i.tags || []).forEach(function (tg) {
          tagCounts[tg] = (tagCounts[tg] || 0) + 1;
        });
      }
    });

    var sortedTags = Object.keys(tagCounts).sort(function (a, b) {
      return tagCounts[b] - tagCounts[a];
    });

    body.innerHTML =
      '<div class="fg">' +
        '<p class="fk">Canonical Tags</p>' +
        '<div class="cs">' +
          sortedTags.map(function (tg) {
            var on = state.tag === tg;
            return '<button type="button" class="chip' + (on ? " on" : "") + '" data-dlg-tag="' + esc(tg) + '">#' + esc(tg) + ' <span class="c">' + tagCounts[tg] + "</span></button>";
          }).join("") +
        "</div>" +
      "</div>";

    dlg.showModal();
  }

  function openSubDialog() {
    var dlg = document.getElementById("sub");
    if (!dlg) return;
    var emailIn = document.getElementById("sub-email");
    var denSel = document.getElementById("sub-density");
    var modeSel = document.getElementById("sub-mode");
    var freqSel = document.getElementById("sub-freq");
    var langSel = document.getElementById("sub-lang");
    var cfiltSel = document.getElementById("sub-cfilter");

    if (emailIn) emailIn.value = state.subPrefs.email || "";
    if (denSel) denSel.value = state.subPrefs.density || "three-per-area";
    if (modeSel) modeSel.value = state.subPrefs.mode || "standard";
    if (freqSel) freqSel.value = state.subPrefs.frequency || "daily";
    if (langSel) langSel.value = state.subPrefs.lang || state.lang;
    if (cfiltSel) cfiltSel.value = state.subPrefs.content_filter || "all";

    renderSubPickers();
    dlg.showModal();
  }

  function renderSubPickers() {
    var ap = document.getElementById("sub-areas-picker");
    var tp = document.getElementById("sub-tags-picker");
    if (ap) {
      ap.innerHTML = AREAS.map(function (a) {
        var on = (state.subPrefs.areas || []).indexOf(a.slug) !== -1;
        return '<button type="button" class="chip' + (on ? " on" : "") + '" data-sub-area="' + esc(a.slug) + '"><span class="rn">' + esc(a.numeral) + "</span> " + esc(areaTitle(a)) + "</button>";
      }).join("");
    }
    if (tp) {
      var tags = [
        "llm", "agents", "ml-theory", "compilers", "distributed-systems", "databases", "rust",
        "formal-proofs", "number-theory", "combinatorics", "quantum-computing", "condensed-matter",
        "kernel-os", "computer-architecture", "infosec", "bioinformatics", "genomics-proteins",
        "zkp", "cryptography", "post-quantum", "macroeconomics", "monetary-policy", "quant-finance",
        "geopolitics", "tech-policy", "defense-strategy", "philosophy-of-science", "epistemology", "decision-theory"
      ];
      tp.innerHTML = tags.map(function (tg) {
        var on = (state.subPrefs.tags || []).indexOf(tg) !== -1;
        return '<button type="button" class="chip' + (on ? " on" : "") + '" data-sub-tag="' + esc(tg) + '">#' + esc(tg) + "</button>";
      }).join("");
    }
  }

  function openCommandPalette() {
    var dlg = document.getElementById("pal");
    var input = document.getElementById("pal-q");
    if (!dlg || !input) return;
    input.placeholder = tr().palPh;
    input.value = "";
    renderPaletteList("");
    dlg.showModal();
    input.focus();
  }

  function renderPaletteList(q) {
    var listEl = document.getElementById("pal-list");
    if (!listEl || !state.issue) return;
    var t = tr();
    var clean = q.toLowerCase().trim();
    var html = "";

    var matchedAreas = AREAS.filter(function (a) {
      return !clean || areaTitle(a).toLowerCase().indexOf(clean) !== -1 || a.slug.indexOf(clean) !== -1;
    });
    if (matchedAreas.length) {
      html += '<li class="pal-g">' + esc(t.gArea) + "</li>";
      matchedAreas.forEach(function (a) {
        html += '<li role="option" data-pal-area="' + esc(a.slug) + '"><span class="rn">' + esc(a.numeral) + '</span> <span class="pl">' + esc(areaTitle(a)) + '</span><span class="ps">' + esc(a.slug) + "</span></li>";
      });
    }

    var matchedItems = (state.issue.items || []).filter(function (i) {
      if (!clean) return i.is_top_pick;
      return itemHeadline(i).toLowerCase().indexOf(clean) !== -1 ||
             i.original_title.toLowerCase().indexOf(clean) !== -1 ||
             (i.tags || []).some(function (tg) { return tg.indexOf(clean) !== -1; });
    }).slice(0, 12);

    if (matchedItems.length) {
      html += '<li class="pal-g">' + esc(t.gRec) + "</li>";
      matchedItems.forEach(function (i) {
        html += '<li role="option" data-pal-item="' + esc(i.id) + '"><span class="pl">' + esc(itemHeadline(i)) + '</span><span class="ps">' + esc(i.source_name) + "</span></li>";
      });
    }

    if (!html) {
      html = '<li class="pal-none">' + esc(t.palNone) + "</li>";
    }
    listEl.innerHTML = html;
  }

  function openKbHelp() {
    var dlg = document.getElementById("kbh");
    var titleEl = document.getElementById("kbh-t");
    var listEl = document.getElementById("kbh-list");
    if (!dlg || !listEl) return;
    var t = tr();
    if (titleEl) titleEl.textContent = t.kbTitle;
    listEl.innerHTML = t.keys.map(function (pair) {
      return "<dt><kbd>" + esc(pair[0]) + "</kbd></dt><dd>" + esc(pair[1]) + "</dd>";
    }).join("");
    dlg.showModal();
  }

  function formatMarkdown(item) {
    var d = item.detail || {};
    var tags = (item.tags || []).map(function (t) { return t.replace(/-/g, "_"); });
    var authorsStr = (item.authors && item.authors.length) ? JSON.stringify(item.authors) : "[]";
    var links = item.links || {};

    var lines = [
      "---",
      "title: " + JSON.stringify(itemHeadline(item)),
      "original_title: " + JSON.stringify(item.original_title || itemHeadline(item)),
      "area: " + JSON.stringify(item.area),
      "source: " + JSON.stringify(item.source_name),
      "url: " + JSON.stringify(item.url),
      "date: " + JSON.stringify(item.issue_date || new Date().toISOString().slice(0, 10)),
      "authors: " + authorsStr,
      "tags: [" + tags.map(function (t) { return '"' + t + '"'; }).join(", ") + "]",
      "score: " + (item.metrics && item.metrics.final_score ? item.metrics.final_score.toFixed(1) : "0.0"),
      item.metrics && item.metrics.citation_count ? "citations: " + item.metrics.citation_count : null,
      d.author_institutions && d.author_institutions.length ? "institutions: " + JSON.stringify(d.author_institutions) : null,
      "---",
      "",
      "# " + itemHeadline(item),
      "",
      "> **" + (state.lang === "tr" ? "Sade Özet" : "Simple Summary") + ":** " + itemSummarySimple(item),
      itemSummaryTechnical(item) && itemSummaryTechnical(item).trim() !== itemSummarySimple(item).trim() ? "> **" + (state.lang === "tr" ? "Teknik Detay" : "Technical Detail") + ":** " + itemSummaryTechnical(item) : null,
      "",
      "- **" + (state.lang === "tr" ? "Kaynak" : "Source") + ":** [" + item.source_name + "](" + item.url + ")",
      links.pdf_url ? "- **PDF:** [" + links.pdf_url + "](" + links.pdf_url + ")" : null,
      links.code_url ? "- **" + (state.lang === "tr" ? "Kod" : "Code") + ":** [" + links.code_url + "](" + links.code_url + ")" : null,
      "",
      "## 1. " + (state.lang === "tr" ? "Problem ve Teknik Bağlam" : "Problem & Technical Context"),
      itemDetailField(item, "problem_context") || itemSummary(item),
      "",
      "## 2. " + (state.lang === "tr" ? "Yöntem ve Mimari" : "Method & Architecture"),
      itemDetailField(item, "method_architecture") || "—",
      "",
      "## 3. " + (state.lang === "tr" ? "Bulgular ve Ödünleşimler" : "Findings & Trade-offs"),
      itemDetailField(item, "findings_tradeoffs") || "—",
      ""
    ];
    if (d.abstract_excerpt) {
      lines.push("## " + (state.lang === "tr" ? "Özet Parçası (Abstract)" : "Abstract Excerpt"));
      lines.push("> " + d.abstract_excerpt);
      lines.push("");
    }
    if (d.bibtex) {
      lines.push("## BibTeX");
      lines.push("```bibtex\n" + d.bibtex + "\n```");
      lines.push("");
    }
    return lines.filter(function (l) { return l !== null; }).join("\n");
  }

  function downloadFile(filename, text, mime) {
    var blob = new Blob([text], { type: mime || "text/plain;charset=utf-8" });
    var url = URL.createObjectURL(blob);
    var a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  function jumpRandom() {
    if (!state.issue || !state.issue.items || !state.issue.items.length) return;
    var items = state.issue.items;
    var pick = items[Math.floor(Math.random() * items.length)];
    state.activeItemId = pick.id;
    state.comparing = false;
    pushUrl(false);
    renderAll();
    window.scrollTo({ top: 0, behavior: "smooth" });
  }


  // --- 5-Minute Daily Audio Briefing (SpeechSynthesis) ---
  var audioState = {
    supported: ("speechSynthesis" in window) && ("SpeechSynthesisUtterance" in window),
    speaking: false,
    paused: false,
    index: 0,
    items: [],
    rate: 1.0,
    utter: null
  };

  function initAudioBrief() {
    if (!audioState.supported || !state.issue || !state.issue.items) return;
    var picks = [];
    AREAS.forEach(function (a) {
      var item = state.issue.items.find(function (i) { return i.area === a.slug && i.is_top_pick; }) ||
                 state.issue.items.find(function (i) { return i.area === a.slug; });
      if (item) picks.push(item);
    });
    audioState.items = picks;
    audioState.index = 0;
  }

  function getSpeechVoice(lang) {
    if (!window.speechSynthesis) return null;
    var voices = window.speechSynthesis.getVoices() || [];
    var prefix = lang === "tr" ? "tr" : "en";
    var found = voices.find(function (v) { return v.lang && v.lang.toLowerCase().indexOf(prefix) === 0; });
    return found || null;
  }

  function speakCurrentItem() {
    if (!audioState.supported || !audioState.items.length) return;
    window.speechSynthesis.cancel();
    if (audioState.index >= audioState.items.length) {
      stopAudioBrief();
      return;
    }
    var item = audioState.items[audioState.index];
    var aMeta = getArea(item.area);
    var head = itemHeadline(item);
    var sum = itemSummary(item);

    var text = (state.lang === "tr")
      ? (aMeta.numeral + ". Kategori: " + areaTitle(aMeta) + ". " + head + ". " + sum)
      : ("Category " + aMeta.numeral + ": " + areaTitle(aMeta) + ". " + head + ". " + sum);

    var utter = new SpeechSynthesisUtterance(text);
    utter.rate = audioState.rate;
    utter.voice = getSpeechVoice(state.lang);
    utter.lang = state.lang === "tr" ? "tr-TR" : "en-US";

    utter.onend = function () {
      if (audioState.speaking && !audioState.paused) {
        audioState.index += 1;
        speakCurrentItem();
      }
    };
    utter.onerror = function () {
      stopAudioBrief();
    };

    audioState.utter = utter;
    audioState.speaking = true;
    audioState.paused = false;
    updateAudioDockUi();
    window.speechSynthesis.speak(utter);
  }

  function toggleAudioBrief() {
    if (!audioState.supported) {
      alert(state.lang === "tr" ? "Tarayıcınız ses sentezleme (SpeechSynthesis) desteklemiyor." : "Speech synthesis not supported in this browser.");
      return;
    }
    if (!audioState.speaking) {
      initAudioBrief();
      if (!audioState.items.length) return;
      audioState.index = 0;
      audioState.speaking = true;
      speakCurrentItem();
    } else if (audioState.paused) {
      window.speechSynthesis.resume();
      audioState.paused = false;
      updateAudioDockUi();
    } else {
      window.speechSynthesis.pause();
      audioState.paused = true;
      updateAudioDockUi();
    }
  }

  function stopAudioBrief() {
    if (window.speechSynthesis) window.speechSynthesis.cancel();
    audioState.speaking = false;
    audioState.paused = false;
    audioState.index = 0;
    updateAudioDockUi();
  }

  function nextAudioItem() {
    if (audioState.index < audioState.items.length - 1) {
      audioState.index += 1;
      speakCurrentItem();
    } else {
      stopAudioBrief();
    }
  }

  function prevAudioItem() {
    if (audioState.index > 0) {
      audioState.index -= 1;
      speakCurrentItem();
    }
  }

  function cycleAudioRate() {
    var rates = [1.0, 1.25, 1.5, 1.75];
    var currIdx = rates.indexOf(audioState.rate);
    audioState.rate = rates[(currIdx + 1) % rates.length];
    if (audioState.speaking && !audioState.paused) {
      speakCurrentItem();
    } else {
      updateAudioDockUi();
    }
  }

  function updateAudioDockUi() {
    var dock = document.getElementById("audio-dock");
    var btn = document.getElementById("audiobtn");
    var adToggle = document.getElementById("ad-toggle");
    var adTitle = document.getElementById("ad-title");
    var adMeta = document.getElementById("ad-meta");
    var adRate = document.getElementById("ad-rate");
    var t = tr();

    if (!dock) return;

    if (!audioState.speaking) {
      dock.style.display = "none";
      if (btn) btn.textContent = t.audioBriefBtn;
      return;
    }

    dock.style.display = "flex";
    if (btn) btn.textContent = audioState.paused ? ("❚❚ " + t.audioPaused) : ("▶ " + t.audioPlaying);
    if (adToggle) adToggle.textContent = audioState.paused ? "▶" : "❚❚";
    if (adRate) adRate.textContent = audioState.rate.toFixed(2).replace(/\.00$/, "") + "×";

    var currItem = audioState.items[audioState.index];
    if (currItem) {
      if (adTitle) adTitle.textContent = itemHeadline(currItem);
      if (adMeta) adMeta.textContent = (audioState.index + 1) + " / " + audioState.items.length;
    }
  }

  // --- Global Events ---
  function bindEvents() {
    var qIn = document.getElementById("q");
    if (qIn) {
      qIn.addEventListener("input", function () {
        state.query = qIn.value;
        state.activeItemId = null;
        state.comparing = false;
        state.savedOnly = false;
        state.sharedShelfIds = [];
        pushUrl(true);
        renderAll();
      });
    }

    var savedBtn = document.getElementById("savedb");
    if (savedBtn) {
      savedBtn.onclick = function () {
        state.savedOnly = !state.savedOnly;
        state.sharedShelfIds = [];
        state.activeItemId = null;
        state.comparing = false;
        pushUrl(false);
        renderAll();
      };
    }

    var subBtn = document.getElementById("subbtn");
    if (subBtn) subBtn.onclick = openSubDialog;

    var randBtn = document.getElementById("rand");
    if (randBtn) randBtn.onclick = jumpRandom;

    var langBtn = document.getElementById("langbtn");
    if (langBtn) {
      langBtn.onclick = function () {
        switchLanguage(state.lang === "en" ? "tr" : "en");
      };
    }

    // Compare tray buttons
    var trayGo = document.getElementById("tray-go");
    var trayClear = document.getElementById("tray-clear");
    if (trayGo) {
      trayGo.onclick = function (e) {
        e.preventDefault();
        state.comparing = true;
        state.activeItemId = null;
        pushUrl(false);
        renderAll();
        window.scrollTo({ top: 0, behavior: "smooth" });
      };
    }
    if (trayClear) {
      trayClear.onclick = function () {
        state.cmpIds = [];
        state.comparing = false;
        pushUrl(true);
        renderAll();
      };
    }

    // Dialog close buttons
    var fClose = document.getElementById("f-close");
    var fClear = document.getElementById("f-clear");
    var filtDlg = document.getElementById("filt");
    if (fClose && filtDlg) fClose.onclick = function () { filtDlg.close(); };
    if (fClear && filtDlg) {
      fClear.onclick = function () {
        state.tag = null;
        state.filters = { code: false, papers: false, discussed: false };
        filtDlg.close();
        pushUrl(false);
        renderAll();
      };
    }

    var sClose = document.getElementById("s-close");
    var subDlg = document.getElementById("sub");
    if (sClose && subDlg) sClose.onclick = function () { subDlg.close(); };

    var kbhClose = document.getElementById("kbh-close");
    var kbhDlg = document.getElementById("kbh");
    if (kbhClose && kbhDlg) kbhClose.onclick = function () { kbhDlg.close(); };

    var palIn = document.getElementById("pal-q");
    if (palIn) {
      palIn.addEventListener("input", function () {
        renderPaletteList(palIn.value);
      });
    }

    // Export buttons in footer
    var expMd = document.getElementById("exp-md");
    if (expMd) {
      expMd.onclick = function () {
        var list = Object.values(state.savedMap);
        if (!list.length && state.issue) list = state.issue.items.filter(function (i) { return i.is_top_pick; });
        var md = "# Ahbar Dispatches\n\n" + list.map(formatMarkdown).join("\n---\n\n");
        downloadFile("ahbar-dispatches.md", md, "text/markdown;charset=utf-8");
      };
    }
    var expBib = document.getElementById("exp-bib");
    if (expBib) {
      expBib.onclick = function () {
        var list = Object.values(state.savedMap);
        if (!list.length && state.issue) list = state.issue.items.filter(function (i) { return i.is_paper; });
        var bib = list.map(function (i) { return i.detail && i.detail.bibtex; }).filter(Boolean).join("\n\n");
        downloadFile("ahbar-citations.bib", bib || "% No BibTeX entries\n", "application/x-bibtex;charset=utf-8");
      };
    }
    var expJson = document.getElementById("exp-json");
    if (expJson) {
      expJson.onclick = function () {
        var list = Object.values(state.savedMap);
        if (!list.length && state.issue) list = state.issue.items;
        downloadFile("ahbar-dispatches.json", JSON.stringify(list, null, 2), "application/json;charset=utf-8");
      };
    }

    // Subscription form submit
    var subForm = document.getElementById("sub-form");
    if (subForm) {
      var isSubmitting = false;
      subForm.addEventListener("submit", function (e) {
        e.preventDefault();
        if (isSubmitting) return;

        var emailRaw = (document.getElementById("sub-email").value || "").trim().toLowerCase();
        if (!emailRaw) return;

        state.subPrefs.email = emailRaw;
        state.subPrefs.density = document.getElementById("sub-density").value;
        state.subPrefs.mode = document.getElementById("sub-mode").value;
        state.subPrefs.frequency = document.getElementById("sub-freq").value;
        state.subPrefs.lang = document.getElementById("sub-lang").value;
        state.subPrefs.content_filter = document.getElementById("sub-cfilter").value;
        saveJson(STORAGE_KEYS.prefs, state.subPrefs);

        var msgEl = document.getElementById("sub-msg");
        var submitBtn = subForm.querySelector('button[type="submit"]');
        if (submitBtn) submitBtn.disabled = true;
        isSubmitting = true;

        if (msgEl) {
          msgEl.textContent = state.lang === "tr"
            ? "✓ Tercihler kaydedildi (" + state.subPrefs.email + "). Tekil bülten dağıtımı otomatik olarak senkronize edildi."
            : "✓ Preferences saved (" + state.subPrefs.email + "). Dispatch delivery synced.";
        }

        var apiUrl = window.AHBAR_API_URL || "";
        var gFormUrl = window.AHBAR_GOOGLE_FORM_URL || "";
        var gEmailField = window.AHBAR_GOOGLE_FORM_EMAIL_FIELD || "entry.email";
        var gLangField = window.AHBAR_GOOGLE_FORM_LANG_FIELD || "entry.lang";

        // Google Form background sink (submits cleanly without leaving the page or CORS errors)
        if (gFormUrl) {
          try {
            var iframeName = "gform_hidden_sink";
            var sink = document.getElementById(iframeName);
            if (!sink) {
              sink = document.createElement("iframe");
              sink.name = iframeName;
              sink.id = iframeName;
              sink.style.display = "none";
              sink.setAttribute("aria-hidden", "true");
              document.body.appendChild(sink);
            }
            var tempForm = document.createElement("form");
            tempForm.action = gFormUrl;
            tempForm.method = "POST";
            tempForm.target = iframeName;
            tempForm.style.display = "none";

            var emInput = document.createElement("input");
            emInput.name = gEmailField;
            emInput.value = emailRaw;
            tempForm.appendChild(emInput);

            if (gLangField && state.subPrefs.lang) {
              var langInput = document.createElement("input");
              langInput.name = gLangField;
              langInput.value = state.subPrefs.lang;
              tempForm.appendChild(langInput);
            }

            document.body.appendChild(tempForm);
            tempForm.submit();
            setTimeout(function () { tempForm.remove(); }, 1500);
          } catch (err) {}
        }

        var req = (apiUrl && typeof fetch === "function")
          ? fetch(apiUrl + "/api/subscribe", {
              method: "POST",
              headers: { "Content-Type": "application/json" },
              body: JSON.stringify(state.subPrefs)
            }).catch(function () {})
          : Promise.resolve();

        req.finally(function () {
          setTimeout(function () {
            isSubmitting = false;
            if (submitBtn) submitBtn.disabled = false;
          }, 800);
        });
      });
    }

    // Delegated clicks
    document.addEventListener("click", function (e) {
      var homeLink = e.target.closest("[data-home]");
      if (homeLink) {
        e.preventDefault();
        state.activeItemId = null;
        state.comparing = false;
        state.showAllList = false;
        state.area = null;
        state.tag = null;
        state.query = "";
        state.savedOnly = false;
        state.sharedShelfIds = [];
        state.filters = { code: false, papers: false, discussed: false };
        if (qIn) qIn.value = "";
        pushUrl(false);
        renderAll();
        window.scrollTo({ top: 0, behavior: "smooth" });
        return;
      }

      var allLink = e.target.closest("[data-all]");
      if (allLink && !e.ctrlKey && !e.metaKey) {
        e.preventDefault();
        state.showAllList = true;
        state.area = null;
        state.tag = null;
        state.activeItemId = null;
        state.comparing = false;
        pushUrl(false);
        renderAll();
        window.scrollTo({ top: 0, behavior: "smooth" });
        return;
      }

      var areaLink = e.target.closest("[data-area]");
      if (areaLink && !e.ctrlKey && !e.metaKey) {
        e.preventDefault();
        state.area = areaLink.getAttribute("data-area");
        state.tag = null;
        state.activeItemId = null;
        state.comparing = false;
        state.showAllList = false;
        state.savedOnly = false;
        state.sharedShelfIds = [];
        pushUrl(false);
        renderAll();
        window.scrollTo({ top: 0, behavior: "smooth" });
        return;
      }

      var areaTagLink = e.target.closest("[data-area-tag]");
      if (areaTagLink && !e.ctrlKey && !e.metaKey) {
        e.preventDefault();
        var parts = areaTagLink.getAttribute("data-area-tag").split(":");
        state.area = parts[0];
        state.tag = parts[1];
        state.activeItemId = null;
        state.comparing = false;
        state.showAllList = false;
        state.savedOnly = false;
        state.sharedShelfIds = [];
        pushUrl(false);
        renderAll();
        window.scrollTo({ top: 0, behavior: "smooth" });
        return;
      }

      var itemLink = e.target.closest("[data-item]");
      if (itemLink && !e.ctrlKey && !e.metaKey) {
        e.preventDefault();
        state.activeItemId = itemLink.getAttribute("data-item");
        state.comparing = false;
        pushUrl(false);
        renderAll();
        window.scrollTo({ top: 0, behavior: "smooth" });
        return;
      }

      var cmpBtn = e.target.closest("[data-cmp]");
      if (cmpBtn) {
        var cid = cmpBtn.getAttribute("data-cmp");
        var cidx = state.cmpIds.indexOf(cid);
        if (cidx >= 0) {
          state.cmpIds.splice(cidx, 1);
        } else {
          if (state.cmpIds.length >= 3) state.cmpIds.shift();
          state.cmpIds.push(cid);
        }
        renderAll();
        return;
      }

      var cmpRem = e.target.closest("[data-cmp-remove]");
      if (cmpRem) {
        var rid = cmpRem.getAttribute("data-cmp-remove");
        state.cmpIds = state.cmpIds.filter(function (x) { return x !== rid; });
        if (state.cmpIds.length === 0) state.comparing = false;
        pushUrl(true);
        renderAll();
        return;
      }

      var qChip = e.target.closest("[data-q]");
      if (qChip) {
        e.preventDefault();
        state.query = qChip.getAttribute("data-q");
        if (qIn) qIn.value = state.query;
        state.activeItemId = null;
        state.comparing = false;
        pushUrl(false);
        renderAll();
        return;
      }

      var tagSpan = e.target.closest("[data-tag]");
      if (tagSpan) {
        var tg = tagSpan.getAttribute("data-tag");
        state.tag = state.tag === tg ? null : tg;
        state.activeItemId = null;
        state.comparing = false;
        pushUrl(false);
        renderAll();
        return;
      }

      var dlgTag = e.target.closest("[data-dlg-tag]");
      if (dlgTag) {
        var dtg = dlgTag.getAttribute("data-dlg-tag");
        state.tag = state.tag === dtg ? null : dtg;
        if (filtDlg) filtDlg.close();
        pushUrl(false);
        renderAll();
        return;
      }

      var clearTag = e.target.closest("[data-clear-tag]");
      if (clearTag) {
        state.tag = null;
        pushUrl(false);
        renderAll();
        return;
      }

      var toggleFilter = e.target.closest("[data-toggle-filter]");
      if (toggleFilter) {
        var fk = toggleFilter.getAttribute("data-toggle-filter");
        state.filters[fk] = !state.filters[fk];
        renderAll();
        return;
      }

      var saveBtnEl = e.target.closest("[data-save]");
      if (saveBtnEl) {
        var sid = saveBtnEl.getAttribute("data-save");
        if (state.savedMap[sid]) {
          delete state.savedMap[sid];
        } else if (state.issue && state.issue.items) {
          var found = state.issue.items.find(function (i) { return i.id === sid; });
          if (found) state.savedMap[sid] = found;
        }
        saveJson(STORAGE_KEYS.saved, state.savedMap);
        renderAll();
        return;
      }

      var voteBtnEl = e.target.closest("[data-vote]");
      if (voteBtnEl) {
        var vid = voteBtnEl.getAttribute("data-item-id");
        var vtype = voteBtnEl.getAttribute("data-vote");
        state.voteMap[vid] = state.voteMap[vid] === vtype ? null : vtype;
        saveJson(STORAGE_KEYS.votes, state.voteMap);
        renderAll();
        return;
      }

      var subAreaBtn = e.target.closest("[data-sub-area]");
      if (subAreaBtn) {
        var sa = subAreaBtn.getAttribute("data-sub-area");
        var idx = state.subPrefs.areas.indexOf(sa);
        if (idx >= 0) state.subPrefs.areas.splice(idx, 1);
        else state.subPrefs.areas.push(sa);
        renderSubPickers();
        return;
      }

      var subTagBtn = e.target.closest("[data-sub-tag]");
      if (subTagBtn) {
        var st = subTagBtn.getAttribute("data-sub-tag");
        var tidx = state.subPrefs.tags.indexOf(st);
        if (tidx >= 0) state.subPrefs.tags.splice(tidx, 1);
        else state.subPrefs.tags.push(st);
        renderSubPickers();
        return;
      }

      var palArea = e.target.closest("[data-pal-area]");
      if (palArea) {
        state.area = palArea.getAttribute("data-pal-area");
        state.activeItemId = null;
        state.comparing = false;
        document.getElementById("pal").close();
        pushUrl(false);
        renderAll();
        return;
      }

      var palItem = e.target.closest("[data-pal-item]");
      if (palItem) {
        state.activeItemId = palItem.getAttribute("data-pal-item");
        state.comparing = false;
        document.getElementById("pal").close();
        pushUrl(false);
        renderAll();
        return;
      }
    });

    // Scroll to top button
    var topBtn = document.getElementById("top");
    if (topBtn) {
      window.addEventListener("scroll", function () {
        topBtn.classList.toggle("show", window.scrollY > 420);
      }, { passive: true });
      topBtn.onclick = function () {
        window.scrollTo({ top: 0, behavior: "smooth" });
      };
    }

    // Popstate
    window.addEventListener("popstate", function () {
      readUrlParams();
      renderAll();
    });

    // Keyboard shortcuts
    document.addEventListener("keydown", function (e) {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        openCommandPalette();
        return;
      }

      var tag = document.activeElement ? document.activeElement.tagName : "";
      var inInput = tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT";

      if (e.key === "Escape") {
        if (inInput && document.activeElement) {
          document.activeElement.blur();
          return;
        }
        if (state.activeItemId || state.comparing || state.showAllList || state.area || state.tag || state.query || state.savedOnly) {
          state.activeItemId = null;
          state.comparing = false;
          state.showAllList = false;
          state.area = null;
          state.tag = null;
          state.query = "";
          state.savedOnly = false;
          state.sharedShelfIds = [];
          if (qIn) qIn.value = "";
          pushUrl(false);
          renderAll();
        }
        return;
      }

      if (inInput) return;

      if (e.key === "/") {
        e.preventDefault();
        if (qIn) qIn.focus();
      } else if (e.key === "?") {
        e.preventDefault();
        openKbHelp();
      } else if (e.key === "r") {
        e.preventDefault();
        jumpRandom();
      } else if (!state.activeItemId && !state.comparing && state.visibleItems.length > 0) {
        var len = state.visibleItems.length;
        if (e.key === "j") {
          e.preventDefault();
          state.kbIndex = Math.min(len - 1, state.kbIndex + 1);
          highlightKb();
        } else if (e.key === "k") {
          e.preventDefault();
          state.kbIndex = Math.max(0, state.kbIndex <= 0 ? 0 : state.kbIndex - 1);
          highlightKb();
        } else if (e.key === "Enter" && state.kbIndex >= 0 && state.kbIndex < len) {
          e.preventDefault();
          state.activeItemId = state.visibleItems[state.kbIndex].id;
          pushUrl(false);
          renderAll();
          window.scrollTo({ top: 0, behavior: "smooth" });
        } else if (e.key === "o" && state.kbIndex >= 0 && state.kbIndex < len) {
          e.preventDefault();
          var targetUrl = safeUrl(state.visibleItems[state.kbIndex].url);
          if (targetUrl !== "#") {
            window.open(targetUrl, "_blank", "noopener");
          }
        } else if (e.key === "s" && state.kbIndex >= 0 && state.kbIndex < len) {
          e.preventDefault();
          var targetId = state.visibleItems[state.kbIndex].id;
          if (state.savedMap[targetId]) delete state.savedMap[targetId];
          else state.savedMap[targetId] = state.visibleItems[state.kbIndex];
          saveJson(STORAGE_KEYS.saved, state.savedMap);
          renderAll();
        }
      }
    });
  }

  function highlightKb() {
    var rows = document.querySelectorAll(".rec");
    rows.forEach(function (r, idx) {
      var on = idx === state.kbIndex;
      r.classList.toggle("kb-focus", on);
      if (on) r.scrollIntoView({ block: "nearest", behavior: "smooth" });
    });
  }

  initTheme();
  bindEvents();
  initData();
})();
