/* Ahbar — Client application.
   Built directly on the useful-sites editorial architecture:
   - Bilingual (EN / TR) UI + bilingual technical headlines, summaries & Layer-2 analyses
   - Home view (body.calm.at-home): Hero, Enso watermark, .proof telemetry, .tryq chips,
     3-column Top Picks (ol.hpicks), 2-column Fihrist Index (ol.fx with .ld dotted leaders),
     and full categorized dispatch list (.recs)
   - Area / Tag / Search / Detail views: 2-column layout (.cols) with sticky 200px left Fihrist (.side)
   - Layer-2 Item Detail Page (?item=<id>): Problem, Method, Findings, Abstract, BibTeX & Markdown copy
   - Command Palette (Ctrl+K / Cmd+K), Keyboard Shortcuts (?, j/k, Enter, o, s, r, /), KaTeX math */

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
      en: "Mathematics",
      tr: "Matematik",
      desc_en: "Number theory, combinatorics, logic, formal proof assistants (Lean 4 / Rocq), algebra and analysis.",
      desc_tr: "Sayılar kuramı, kombinatorik, mantık, biçimsel kanıt asistanları (Lean 4 / Rocq), cebir ve analiz.",
      tags: ["number-theory", "combinatorics", "formal-proofs", "analysis-pde", "algebra-geometry", "probability-stats", "logic"]
    },
    {
      slug: "physics",
      numeral: "IV",
      en: "Physics & Quantum",
      tr: "Fizik & Kuantum",
      desc_en: "Quantum computing, condensed matter, astrophysics, high-energy theory and optics.",
      desc_tr: "Kuantum hesaplama, yoğun madde, astrofizik, yüksek enerji kuramı ve optik/fotonik.",
      tags: ["quantum-computing", "quantum-physics", "condensed-matter", "high-energy", "astrophysics", "optics-photonics"]
    },
    {
      slug: "systems",
      numeral: "V",
      en: "Systems, Hardware & Security",
      tr: "Sistem, Donanım & Güvenlik",
      desc_en: "Operating systems, kernels, semiconductors, computer architecture, networking and InfoSec.",
      desc_tr: "İşletim sistemleri, çekirdek, yarı iletkenler, bilgisayar mimarisi, ağ protokolleri ve siber güvenlik.",
      tags: ["kernel-os", "computer-architecture", "semiconductors", "infosec", "networking", "performance"]
    },
    {
      slug: "bio",
      numeral: "VI",
      en: "Computational Science & Bio",
      tr: "Hesaplamalı Bilim & Biyoloji",
      desc_en: "Bioinformatics, computational neuroscience, genomics, protein design, materials science and clean tech.",
      desc_tr: "Biyoinformatik, hesaplamalı sinirbilim, genomik, protein tasarımı, malzeme bilimi ve temiz enerji.",
      tags: ["bioinformatics", "genomics-proteins", "neuroscience", "materials-science", "clean-tech"]
    },
    {
      slug: "crypto",
      numeral: "VII",
      en: "Cryptography, Web3 & Blockchain",
      tr: "Kriptografi, Web3 & Blokzincir",
      desc_en: "Zero-knowledge proofs, post-quantum cryptography, consensus protocols, smart contract security and MEV.",
      desc_tr: "Sıfır-bilgi kanıtları (ZKP), kuantum-sonrası kriptografi, mutabakat protokolleri ve akıllı sözleşme güvenliği.",
      tags: ["zkp", "cryptography", "post-quantum", "consensus", "smart-contracts"]
    }
  ];

  var T = {
    en: {
      title: "Ahbar",
      hero: function (n, issueNo) {
        return "<em>" + n + "</em> technical &amp; scientific dispatches in Issue #" + issueNo + ". Each one states what was proved, built, or measured.";
      },
      proof: function (n, date, sources, scanned) {
        return n + " qualified dispatches · Issue " + date + " · " + scanned + " raw papers &amp; feeds triaged from " + sources + " primary sources · updated 05:00 UTC · no trackers";
      },
      tryLbl: "Try:",
      tryQs: ["llm", "rust", "zkp", "quantum", "lean 4", "kernel", "protein"],
      hStart: "Today's Top Picks",
      hStartNote: "3 per category · 21 dispatches",
      fxHead: "Index",
      fxNote: function (a, t) { return a + " areas, " + t + " canonical tags"; },
      allDispatches: "All Dispatches by Category",
      allAreas: "← Full Index (Home)",
      sideHead: "Index",
      saved: "Saved",
      emailPrev: "Email (7×3) ↗",
      emailHref: "data/sample-email.html",
      subscribe: "Subscribe",
      rand: "Random",
      langBtn: "TR",
      langTitle: "Türkçeye geç",
      ph: "Search papers, theorems, systems, #tags…",
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
      saveTip: "Save to local reading list (s)",
      savedEmpty: "Nothing saved yet. Click ☆ or press s on any dispatch to keep it here for Markdown (.md) or BibTeX (.bib) export.",
      backArea: "← All dispatches in this area",
      origTitle: "Original Title",
      authorsLbl: "Authors",
      openSource: "Open Primary Source ↗",
      readPdf: "Read PDF ↗",
      readHtml: "Read HTML (ar5iv) ↗",
      openCode: "Code Repository ↗",
      copyBib: "Copy BibTeX",
      copyMd: "Copy Markdown",
      copied: "✓ Copied",
      sec1: "I. Problem & Technical Context",
      sec2: "II. Method, Architecture & Formal Construction",
      sec3: "III. Empirical Findings, Benchmarks & Trade-offs",
      absHead: "Original Paper Abstract / Excerpt",
      clusteredHead: "Clustered Sources & Discussions",
      relatedHead: "Related Dispatches in This Issue",
      footText: function (date, sources) {
        return "Autonomous daily technical &amp; scientific digest, last built <b>" + date + "</b> at <code>05:00 UTC</code> (08:00 TSİ) across <b>" + sources + "</b> primary feeds. Zero marketing copy; every dispatch states what was proved, built, or measured. <a href=\"data/health.json\" target=\"_blank\" rel=\"noopener\">Source health telemetry ↗</a>";
      },
      expLbl: "Export saved:",
      kbFoot: "<kbd>/</kbd> Search · <kbd>Ctrl K</kbd> Commands · <kbd>j</kbd>/<kbd>k</kbd> Navigate · <kbd>Enter</kbd> Analysis · <kbd>s</kbd> Save · <kbd>r</kbd> Random · <kbd>?</kbd> Shortcuts",
      palPh: "Jump to a dispatch, area, #tag, or command…",
      palNone: "Nothing matches",
      gAct: "Actions",
      gArea: "Areas",
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
        return "Sayı #" + issueNo + " içinde <em>" + n + "</em> teknik ve bilimsel bülten kaydı. Her biri neyin kanıtlandığını, inşa edildiğini veya ölçüldüğünü söyler.";
      },
      proof: function (n, date, sources, scanned) {
        return n + " seçilmiş kayıt · " + date + " · " + sources + " birincil kaynaktan " + scanned + " makale ve besleme tarandı · her gün 08:00 TSİ · izleyici yok";
      },
      tryLbl: "Dene:",
      tryQs: ["llm", "rust", "zkp", "quantum", "lean 4", "kernel", "protein"],
      hStart: "Günün Seçkisi (Top Picks)",
      hStartNote: "her kategoriden 3 haber · toplam 21 kayıt",
      fxHead: "Fihrist",
      fxNote: function (a, t) { return a + " alan, " + t + " kanonik etiket"; },
      allDispatches: "Kategorilere Göre Tüm Kayıtlar",
      allAreas: "← Fihrist (Ana Sayfa)",
      sideHead: "Fihrist",
      saved: "Kaydedilenler",
      emailPrev: "E-Posta (7×3) ↗",
      emailHref: "data/sample-email-tr.html",
      subscribe: "Abone Ol",
      rand: "Rastgele",
      langBtn: "EN",
      langTitle: "Switch to English",
      ph: "Ara: makale, teorem, sistem, #etiket, kaynak…",
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
      saveTip: "Okuma listesine kaydet (s)",
      savedEmpty: "Henüz kaydedilmiş bir haber yok. Herhangi bir kayıttaki ☆ düğmesiyle veya s tuşuyla buraya ekleyip Markdown (.md) ya da BibTeX (.bib) olarak dışa aktarabilirsin.",
      backArea: "← Bu alandaki tüm kayıtlar",
      origTitle: "Orijinal Başlık",
      authorsLbl: "Yazarlar",
      openSource: "Birincil Kaynağı Aç ↗",
      readPdf: "PDF Oku ↗",
      readHtml: "HTML Oku (ar5iv) ↗",
      openCode: "Kod Deposu ↗",
      copyBib: "BibTeX Kopyala",
      copyMd: "Markdown Kopyala",
      copied: "✓ Kopyalandı",
      sec1: "I. Problem & Teknik Bağlam",
      sec2: "II. Yöntem, Mimari & Biçimsel Yapı",
      sec3: "III. Deneysel Bulgular, Kıyaslamalar & Ödünleşimler",
      absHead: "Orijinal Makale Özeti (Abstract)",
      clusteredHead: "Kümelenmiş Kaynaklar & Tartışmalar",
      relatedHead: "Bu Sayıdaki İlgili Diğer Kayıtlar",
      footText: function (date, sources) {
        return "Otonom günlük teknik ve bilimsel bülten; son derleme <b>" + date + "</b> saat <code>08:00 TSİ</code> (<code>05:00 UTC</code>), <b>" + sources + "</b> birincil kaynak tarandı. Pazarlama dili içermez; her kayıt neyin kanıtlandığını veya inşa edildiğini belirtir. <a href=\"data/health.json\" target=\"_blank\" rel=\"noopener\">Kaynak sağlık telemetrisi ↗</a>";
      },
      expLbl: "Kaydedilenleri dışa aktar:",
      kbFoot: "<kbd>/</kbd> Ara · <kbd>Ctrl K</kbd> Komutlar · <kbd>j</kbd>/<kbd>k</kbd> Gezin · <kbd>Enter</kbd> Analiz · <kbd>s</kbd> Kaydet · <kbd>r</kbd> Rastgele · <kbd>?</kbd> Kısayollar",
      palPh: "Bir kayda, alana, #etikete ya da komuta git…",
      palNone: "Eşleşen sonuç yok",
      gAct: "Komutlar",
      gArea: "Alanlar",
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

  var state = {
    lang: loadStr(STORAGE_KEYS.lang, "en"),
    issue: null,
    archive: [],
    canonicalTags: [],
    area: null,
    tag: null,
    query: "",
    sort: "score",
    savedOnly: false,
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
      areas: ["cs", "ai", "math", "physics", "systems", "bio", "crypto"],
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

  function tr() {
    return T[state.lang] || T.en;
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

  // --- Theme & Language Init ---
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

  function triggerMath(el) {
    if (typeof window.renderMathInElement === "function") {
      try {
        window.renderMathInElement(el || document.body, {
          delimiters: [
            { left: "$$", right: "$$", display: true },
            { left: "$", right: "$", display: false },
            { left: "\\(", right: "\\)", display: false },
            { left: "\\[", right: "\\]", display: true }
          ],
          throwOnError: false
        });
      } catch (_) {}
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
    state.savedOnly = p.get("saved") === "1";

    var qInput = document.getElementById("q");
    if (qInput) qInput.value = state.query;

    if (p.get("prefs") === "1") {
      var em = p.get("email");
      if (em) state.subPrefs.email = em;
      setTimeout(openSubDialog, 120);
    }
    if (p.get("unsub") === "1") {
      var uem = p.get("email") || state.subPrefs.email || "";
      setTimeout(function () {
        alert(state.lang === "tr"
          ? uem + " adresi günlük e-posta listesinden çıkarıldı."
          : "One-click unsubscribe completed for " + uem + ".");
      }, 120);
    }
  }

  function pushUrl(replace) {
    var p = new URLSearchParams();
    if (state.lang === "tr") p.set("lang", "tr");
    if (state.issue && state.archive.length > 0 && state.issue.date !== state.archive[0].date) {
      p.set("issue", state.issue.date);
    }
    if (state.activeItemId) {
      p.set("item", state.activeItemId);
    } else {
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
      !state.area &&
      !state.tag &&
      !state.query.trim() &&
      !state.savedOnly &&
      !state.filters.code &&
      !state.filters.papers &&
      !state.filters.discussed
    );
  }

  // --- Data Loading ---
  function fetchJson(path) {
    return fetch(path, { cache: "no-cache" }).then(function (r) {
      if (!r.ok) throw new Error("HTTP " + r.status);
      return r.json();
    });
  }

  function initData() {
    readUrlParams();
    var p = new URLSearchParams(window.location.search);
    var reqIssue = p.get("issue");

    Promise.allSettled([
      fetchJson("data/archive.json"),
      fetchJson("data/tags.json"),
      reqIssue ? fetchJson("data/issues/" + reqIssue + ".json") : fetchJson("data/latest.json")
    ]).then(function (res) {
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
    fetchJson("data/issues/" + dateStr + ".json").then(function (iss) {
      state.issue = iss;
      state.activeItemId = null;
      pushUrl(false);
      renderAll();
    });
  }

  // --- Filtering ---
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
      var q = state.query.toLowerCase().trim();
      var cleanTag = q.charAt(0) === "#" ? q.slice(1) : q;
      if (q.charAt(0) === "#") {
        return (item.tags || []).some(function (t) { return t.toLowerCase().indexOf(cleanTag) !== -1; });
      }
      var hay = [
        item.headline,
        item.summary,
        item.headline_tr || "",
        item.summary_tr || "",
        item.original_title,
        item.source_name,
        (item.authors || []).join(" "),
        (item.tags || []).join(" ")
      ].join(" ").toLowerCase();
      if (hay.indexOf(q) === -1) return false;
    }
    return true;
  }

  function getFilteredItems() {
    if (!state.issue || !state.issue.items) return [];
    if (state.savedOnly) {
      return Object.values(state.savedMap);
    }
    var list = state.issue.items.filter(function (i) { return matchesItem(i, false, false); });
    list.sort(function (a, b) {
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

  // --- Render Chrome (Header, Sidebar, Footer) ---
  function renderChrome() {
    var t = tr();
    document.documentElement.lang = state.lang;

    var atHome = isAtHome();
    document.body.classList.toggle("at-home", atHome);
    document.body.classList.toggle("calm", atHome);

    var totalItems = state.issue && state.issue.items ? state.issue.items.length : 63;
    var issueNo = state.issue ? state.issue.issue_number : 1;

    var heroEl = document.getElementById("hero");
    if (heroEl) heroEl.innerHTML = t.hero(totalItems, issueNo);

    var qEl = document.getElementById("q");
    if (qEl) qEl.placeholder = t.ph;

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

    var subBtn = document.getElementById("subbtn");
    if (subBtn) subBtn.textContent = t.subscribe;

    var randBtn = document.getElementById("rand");
    if (randBtn) randBtn.textContent = t.rand;

    var langBtn = document.getElementById("langbtn");
    if (langBtn) {
      langBtn.textContent = t.langBtn;
      langBtn.title = t.langTitle;
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
      var sCount = state.issue.stats ? state.issue.stats.sources_checked : 56;
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
    var vote = state.voteMap[item.id] || "";
    var host = extractHost(item.url);
    var numStr = String(seqIndex).padStart(2, "0");

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
    metaLinks.push('<a href="' + esc(item.url) + '" target="_blank" rel="noopener" class="arch">' + esc(t.sourceExt) + "</a>");
    if (links.pdf_url) {
      metaLinks.push('<a href="' + esc(links.pdf_url) + '" target="_blank" rel="noopener" class="arch">PDF</a>');
    }
    if (links.html_ar5iv_url) {
      metaLinks.push('<a href="' + esc(links.html_ar5iv_url) + '" target="_blank" rel="noopener" class="arch">ar5iv</a>');
    }
    if (links.code_url) {
      metaLinks.push('<a href="' + esc(links.code_url) + '" target="_blank" rel="noopener" class="arch">Code</a>');
    }
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
              esc(itemHeadline(item)) +
            "</a>" +
            (host ? '<span class="host">' + esc(host) + "</span>" : "") +
            badges +
            '<button type="button" class="save" data-save="' + esc(item.id) + '" aria-pressed="' + (isSaved ? "true" : "false") + '" title="' + esc(t.saveTip) + '">' +
              (isSaved ? "★" : "☆") +
            "</button>" +
          "</div>" +
          '<p class="desc">' + esc(itemSummary(item)) + "</p>" +
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

    // 2. Top Picks (1 from each of the first 6 areas in the 3-column ol.hpicks showcase)
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
            esc(itemHeadline(item)) +
          "</a>" +
          "<p>" + esc(itemSummary(item)) + "</p>" +
          '<div class="n" style="margin-top:6px;"><span class="rn">' + esc(aMeta.numeral) + "</span> · " + esc(areaTitle(aMeta)) + " · " + esc(item.source_name) + "</div>" +
        "</li>"
      );
    }).join("");

    // 3. Fihrist (Index: 7 Areas in 2-column ol.fx with dotted leaders .ld and clickable sub-tags)
    var fxHtml = AREAS.map(function (a) {
      var cnt = allItems.filter(function (i) { return i.area === a.slug; }).length;
      var subTags = a.tags.map(function (tg) {
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

    // 4. All Dispatches grouped by the 7 Categories (with Toolbar)
    var globalSeq = 1;
    var categorizedListHtml = "";
    state.visibleItems = [];

    AREAS.forEach(function (a) {
      var areaItems = allItems.filter(function (i) { return i.area === a.slug; });
      if (!areaItems.length) return;
      categorizedListHtml +=
        '<section id="sec-' + esc(a.slug) + '">' +
          '<h3 class="sh"><span class="rn">' + esc(a.numeral) + "</span> " +
            '<a href="?area=' + esc(a.slug) + '" data-area="' + esc(a.slug) + '" style="text-decoration:none;">' + esc(areaTitle(a)) + "</a>" +
            ' <span class="n">' + areaItems.length + "</span>" +
          "</h3>" +
          '<div class="recs">' +
            areaItems.map(function (item) {
              state.visibleItems.push(item);
              return buildRecHtml(item, globalSeq++, false);
            }).join("") +
          "</div>" +
        "</section>";
    });

    mainEl.innerHTML =
      '<div class="home">' +
        '<p class="proof">' + t.proof(allItems.length, state.issue.date, stats.sources_checked || 56, stats.raw_items_scanned || 180) + "</p>" +
        '<p class="tryq"><span>' + esc(t.tryLbl) + "</span>" + tryChips + "</p>" +
        '<section class="hsec hp">' +
          '<h2 class="k">' + esc(t.hStart) + "<span>" + esc(t.hStartNote) + "</span></h2>" +
          '<ol class="hpicks">' + hpicksHtml + "</ol>" +
        "</section>" +
        '<section class="hsec">' +
          '<h2 class="k">' + esc(t.fxHead) + "<span>" + esc(t.fxNote(7, 41)) + "</span></h2>" +
          '<ol class="fx">' + fxHtml + "</ol>" +
        "</section>" +
        '<section class="hsec">' +
          '<h2 class="k">' + esc(t.allDispatches) + "</h2>" +
          buildToolbarHtml(allItems.length, allItems.length) +
          categorizedListHtml +
        "</section>" +
      "</div>";
  }

  function renderFilteredView(mainEl) {
    var t = tr();
    var allItems = state.issue.items || [];
    var filtered = getFilteredItems();
    state.visibleItems = filtered;

    var headerHtml = "";
    if (state.savedOnly) {
      headerHtml =
        '<p class="crumb"><a href="./" data-home="1">' + esc(t.fxHead) + "</a><b>›</b>" + esc(t.saved) + "</p>" +
        '<h1 class="ph">' + esc(t.saved) + ' <span class="n">' + filtered.length + "</span></h1>";
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
  }

  function renderDetailView(mainEl, item) {
    var t = tr();
    state.seenMap[item.id] = Date.now();
    saveJson(STORAGE_KEYS.seen, state.seenMap);

    var aMeta = getArea(item.area);
    var isSaved = !!state.savedMap[item.id];
    var links = item.links || {};
    var detail = item.detail || {};
    var host = extractHost(item.url);

    var visitBtns = [];
    visitBtns.push('<a class="btn primary" href="' + esc(item.url) + '" target="_blank" rel="noopener">' + esc(t.openSource) + "</a>");
    if (links.pdf_url) {
      visitBtns.push('<a class="btn" href="' + esc(links.pdf_url) + '" target="_blank" rel="noopener">' + esc(t.readPdf) + "</a>");
    }
    if (links.html_ar5iv_url) {
      visitBtns.push('<a class="btn" href="' + esc(links.html_ar5iv_url) + '" target="_blank" rel="noopener">' + esc(t.readHtml) + "</a>");
    }
    if (links.code_url) {
      visitBtns.push('<a class="btn" href="' + esc(links.code_url) + '" target="_blank" rel="noopener">' + esc(t.openCode) + "</a>");
    }
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
                '<a href="' + esc(cs.url) + '" target="_blank" rel="noopener">' + esc(cs.source_name) + " ↗</a>" +
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
                '<a href="?item=' + encodeURIComponent(r.id) + '" data-item="' + esc(r.id) + '">' + esc(itemHeadline(r)) + "</a>" +
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
          '<div class="rb">' +
            '<div class="nm">' +
              '<h1 class="ph"><a class="name" href="' + esc(item.url) + '" target="_blank" rel="noopener">' + esc(itemHeadline(item)) + "</a></h1>" +
              '<button type="button" class="save" data-save="' + esc(item.id) + '" aria-pressed="' + (isSaved ? "true" : "false") + '" title="' + esc(t.saveTip) + '">' +
                (isSaved ? "★" : "☆") +
              "</button>" +
            "</div>" +
            '<p class="path" style="margin-top:4px;">' +
              "<b>" + esc(t.origTitle) + ":</b> " + esc(item.original_title) +
              (host ? ' · <span class="host">' + esc(host) + "</span>" : "") +
              (item.authors && item.authors.length ? "<br><b>" + esc(t.authorsLbl) + ":</b> " + esc(item.authors.join(", ")) : "") +
            "</p>" +
            '<p class="desc">' + esc(itemSummary(item)) + "</p>" +
            '<div class="visit">' + visitBtns.join("") + "</div>" +
            '<div class="mt" style="margin-bottom:28px;">' +
              '<span class="itags">' + tagsHtml + "</span>" +
              '<span class="meta"><span class="n">Score ' + item.metrics.final_score.toFixed(1) + " · Novelty " + item.metrics.llm_novelty.toFixed(1) + " · Rigor " + item.metrics.llm_rigor.toFixed(1) + "</span></span>" +
            "</div>" +

            '<h2 class="k">' + esc(t.sec1) + "</h2>" +
            '<p style="margin:10px 0 24px;max-width:70ch;font-size:15.5px;line-height:1.66;color:var(--fg);">' +
              esc(itemDetailField(item, "problem_context")) +
            "</p>" +

            '<h2 class="k">' + esc(t.sec2) + "</h2>" +
            '<p style="margin:10px 0 24px;max-width:70ch;font-size:15.5px;line-height:1.66;color:var(--fg);">' +
              esc(itemDetailField(item, "method_architecture")) +
            "</p>" +

            '<h2 class="k">' + esc(t.sec3) + "</h2>" +
            '<p style="margin:10px 0 24px;max-width:70ch;font-size:15.5px;line-height:1.66;color:var(--fg);">' +
              esc(itemDetailField(item, "findings_tradeoffs")) +
            "</p>" +

            (detail.abstract_excerpt
              ? '<details style="margin:18px 0 24px;padding:12px 0;border-top:1px solid var(--rule2);border-bottom:1px solid var(--rule2);">' +
                  '<summary style="cursor:pointer;font-weight:600;color:var(--dim);">' + esc(t.absHead) + "</summary>" +
                  '<p style="margin:10px 0 0;max-width:70ch;font-size:14.5px;line-height:1.62;color:var(--dim);">' + esc(detail.abstract_excerpt) + "</p>" +
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

    if (state.activeItemId) {
      var found = (state.issue.items || []).find(function (i) { return i.id === state.activeItemId; }) ||
                  state.savedMap[state.activeItemId];
      if (found) {
        renderDetailView(mainEl, found);
        bindToolbarEvents();
        triggerMath(mainEl);
        return;
      } else {
        state.activeItemId = null;
      }
    }

    if (isAtHome()) {
      renderHomeView(mainEl);
    } else {
      renderFilteredView(mainEl);
    }

    bindToolbarEvents();
    triggerMath(mainEl);
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
      var tags = ["llm", "agents", "ml-theory", "compilers", "distributed-systems", "databases", "rust", "formal-proofs", "number-theory", "combinatorics", "quantum-computing", "condensed-matter", "kernel-os", "computer-architecture", "infosec", "bioinformatics", "genomics-proteins", "zkp", "cryptography", "post-quantum", "consensus"];
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

    // Areas matching
    var matchedAreas = AREAS.filter(function (a) {
      return !clean || areaTitle(a).toLowerCase().indexOf(clean) !== -1 || a.slug.indexOf(clean) !== -1;
    });
    if (matchedAreas.length) {
      html += '<li class="pal-g">' + esc(t.gArea) + "</li>";
      matchedAreas.forEach(function (a) {
        html += '<li role="option" data-pal-area="' + esc(a.slug) + '"><span class="rn">' + esc(a.numeral) + '</span> <span class="pl">' + esc(areaTitle(a)) + '</span><span class="ps">' + esc(a.slug) + "</span></li>";
      });
    }

    // Dispatches matching
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
    return [
      "### " + itemHeadline(item),
      "- **Original Title:** " + item.original_title,
      "- **Source:** [" + item.source_name + "](" + item.url + ") (" + item.issue_date + ")",
      item.links && item.links.pdf_url ? "- **PDF:** " + item.links.pdf_url : null,
      item.links && item.links.code_url ? "- **Code:** " + item.links.code_url : null,
      "- **Tags:** " + (item.tags || []).map(function (t) { return "#" + t; }).join(" "),
      "",
      itemSummary(item),
      ""
    ].filter(Boolean).join("\n");
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
    pushUrl(false);
    renderAll();
    window.scrollTo({ top: 0, behavior: "smooth" });
  }

  // --- Global Events ---
  function bindEvents() {
    // Search input
    var qIn = document.getElementById("q");
    if (qIn) {
      qIn.addEventListener("input", function () {
        state.query = qIn.value;
        state.activeItemId = null;
        state.savedOnly = false;
        pushUrl(true);
        renderAll();
      });
    }

    // Top bar actions
    var savedBtn = document.getElementById("savedb");
    if (savedBtn) {
      savedBtn.onclick = function () {
        state.savedOnly = !state.savedOnly;
        state.activeItemId = null;
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
        state.lang = state.lang === "en" ? "tr" : "en";
        saveStr(STORAGE_KEYS.lang, state.lang);
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
      subForm.addEventListener("submit", function (e) {
        e.preventDefault();
        state.subPrefs.email = document.getElementById("sub-email").value.trim();
        state.subPrefs.density = document.getElementById("sub-density").value;
        state.subPrefs.mode = document.getElementById("sub-mode").value;
        state.subPrefs.frequency = document.getElementById("sub-freq").value;
        state.subPrefs.lang = document.getElementById("sub-lang").value;
        state.subPrefs.content_filter = document.getElementById("sub-cfilter").value;
        saveJson(STORAGE_KEYS.prefs, state.subPrefs);

        var msgEl = document.getElementById("sub-msg");
        if (msgEl) {
          msgEl.textContent = state.lang === "tr"
            ? "✓ Tercihler kaydedildi (" + state.subPrefs.density + " · " + state.subPrefs.areas.length + " alan)."
            : "✓ Preferences saved (" + state.subPrefs.density + " · " + state.subPrefs.areas.length + " areas).";
        }
      });
    }

    // Delegated clicks
    document.addEventListener("click", function (e) {
      var homeLink = e.target.closest("[data-home]");
      if (homeLink) {
        e.preventDefault();
        state.activeItemId = null;
        state.area = null;
        state.tag = null;
        state.query = "";
        state.savedOnly = false;
        state.filters = { code: false, papers: false, discussed: false };
        if (qIn) qIn.value = "";
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
        state.savedOnly = false;
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
        state.savedOnly = false;
        pushUrl(false);
        renderAll();
        window.scrollTo({ top: 0, behavior: "smooth" });
        return;
      }

      var itemLink = e.target.closest("[data-item]");
      if (itemLink && !e.ctrlKey && !e.metaKey) {
        e.preventDefault();
        state.activeItemId = itemLink.getAttribute("data-item");
        pushUrl(false);
        renderAll();
        window.scrollTo({ top: 0, behavior: "smooth" });
        return;
      }

      var qChip = e.target.closest("[data-q]");
      if (qChip) {
        e.preventDefault();
        state.query = qChip.getAttribute("data-q");
        if (qIn) qIn.value = state.query;
        state.activeItemId = null;
        pushUrl(false);
        renderAll();
        return;
      }

      var tagSpan = e.target.closest("[data-tag]");
      if (tagSpan) {
        var tg = tagSpan.getAttribute("data-tag");
        state.tag = state.tag === tg ? null : tg;
        state.activeItemId = null;
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
        document.getElementById("pal").close();
        pushUrl(false);
        renderAll();
        return;
      }

      var palItem = e.target.closest("[data-pal-item]");
      if (palItem) {
        state.activeItemId = palItem.getAttribute("data-pal-item");
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
        if (state.activeItemId || state.area || state.tag || state.query || state.savedOnly) {
          state.activeItemId = null;
          state.area = null;
          state.tag = null;
          state.query = "";
          state.savedOnly = false;
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
      } else if (!state.activeItemId && state.visibleItems.length > 0) {
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
          window.open(state.visibleItems[state.kbIndex].url, "_blank", "noopener");
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

  window.addEventListener("load", function () {
    triggerMath(document.body);
  });

  initTheme();
  bindEvents();
  initData();
})();
