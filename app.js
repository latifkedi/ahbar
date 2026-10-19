/* Ahbar — Zero-dependency client runtime.
   Handles:
   - Issue loading (data/latest.json, data/issues/<date>.json, data/archive.json)
   - 2-Layer Navigation: Feed View + Layer-2 Technical Analysis Page (?item=<id>)
   - 7 Top-Level Scientific & Technical Areas + Canonical Tag Filtering
   - Quick Technical Filters ([Has Code], [Papers Only], [Discussed], [Unread Only])
   - KaTeX inline & display LaTeX rendering ($...$, $$...$$)
   - LocalStorage state: Read tracking, Saved Reading List (.md & .bib export),
     Signal/Noise feedback voting, and Passwordless Subscription Preferences
   - Full keyboard navigation (j/k, Enter, o, s, /, Esc) */

(function () {
  "use strict";

  const AREAS = [
    { slug: "all", numeral: "ALL", title: "All Areas", short: "All Areas" },
    { slug: "cs", numeral: "I", title: "Computer Science & Software", short: "CS & Software" },
    { slug: "ai", numeral: "II", title: "AI & Machine Learning", short: "AI & ML" },
    { slug: "math", numeral: "III", title: "Mathematics", short: "Mathematics" },
    { slug: "physics", numeral: "IV", title: "Physics & Quantum", short: "Physics & Quantum" },
    { slug: "systems", numeral: "V", title: "Systems, Hardware & Security", short: "Systems & Security" },
    { slug: "bio", numeral: "VI", title: "Computational Science & Bio", short: "CompBio & Science" },
    { slug: "crypto", numeral: "VII", title: "Cryptography, Web3 & Blockchain", short: "Cryptography & Web3" }
  ];

  const STORAGE_KEYS = {
    theme: "ahbar.theme.v1",
    saved: "ahbar.saved.v1",
    read: "ahbar.read.v1",
    votes: "ahbar.votes.v1",
    prefs: "ahbar.sub_prefs.v1"
  };

  const state = {
    issue: null,
    archive: [],
    canonicalTags: [],
    area: "all",
    tag: null,
    query: "",
    sort: "score",
    filters: {
      code: false,
      papers: false,
      discussed: false,
      unread: false
    },
    activeItemId: null,
    kbIndex: -1,
    visibleItems: [],
    savedMap: loadJson(STORAGE_KEYS.saved, {}),
    readMap: loadJson(STORAGE_KEYS.read, {}),
    voteMap: loadJson(STORAGE_KEYS.votes, {}),
    subPrefs: loadJson(STORAGE_KEYS.prefs, {
      email: "",
      mode: "hybrid",
      density: "compact",
      frequency: "daily",
      content_filter: "all",
      areas: ["cs", "ai", "math", "physics", "systems"],
      tags: []
    })
  };

  function loadJson(key, fallback) {
    try {
      const raw = localStorage.getItem(key);
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

  function escapeHtml(str) {
    return String(str == null ? "" : str)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
  }

  function getAreaMeta(slug) {
    return AREAS.find((a) => a.slug === slug) || AREAS[0];
  }

  // --- Theme Management ---
  function initTheme() {
    const savedTheme = localStorage.getItem(STORAGE_KEYS.theme);
    if (savedTheme === "dark" || savedTheme === "light") {
      document.documentElement.setAttribute("data-theme", savedTheme);
    }
    const btn = document.getElementById("theme-btn");
    if (btn) {
      btn.addEventListener("click", () => {
        const current = document.documentElement.getAttribute("data-theme");
        const prefersDark = window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches;
        const effective = current || (prefersDark ? "dark" : "light");
        const next = effective === "dark" ? "light" : "dark";
        document.documentElement.setAttribute("data-theme", next);
        try {
          localStorage.setItem(STORAGE_KEYS.theme, next);
        } catch (_) {}
      });
    }
  }

  // --- KaTeX Math Rendering ---
  function triggerMathRender(rootEl) {
    const target = rootEl || document.body;
    if (typeof window.renderMathInElement === "function") {
      try {
        window.renderMathInElement(target, {
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

  // --- URL State Sync ---
  function readUrlParams() {
    const params = new URLSearchParams(window.location.search);
    const item = params.get("item");
    const area = params.get("area");
    const tag = params.get("tag");
    const q = params.get("q");
    state.activeItemId = item || null;
    if (area && AREAS.some((a) => a.slug === area)) {
      state.area = area;
    }
    if (tag) {
      state.tag = tag;
    }
    if (q) {
      state.query = q;
      const qInput = document.getElementById("q");
      if (qInput) qInput.value = q;
    }

    if (params.get("prefs") === "1") {
      const emailParam = params.get("email");
      if (emailParam) {
        state.subPrefs.email = emailParam;
      }
      setTimeout(() => openSubModal(), 150);
    }

    if (params.get("unsub") === "1") {
      const emailParam = params.get("email") || state.subPrefs.email || "your address";
      setTimeout(() => {
        alert("One-click unsubscribe processed for " + emailParam + ". You have been removed from the daily dispatch.");
      }, 150);
    }
  }

  function pushUrlState(replace) {
    const params = new URLSearchParams();
    if (state.issue && state.archive.length > 0 && state.issue.date !== state.archive[0].date) {
      params.set("issue", state.issue.date);
    }
    if (state.activeItemId) {
      params.set("item", state.activeItemId);
    } else {
      if (state.area && state.area !== "all") params.set("area", state.area);
      if (state.tag) params.set("tag", state.tag);
      if (state.query) params.set("q", state.query);
    }
    const qs = params.toString();
    const nextUrl = qs ? "?" + qs : window.location.pathname;
    if (replace) {
      window.history.replaceState(null, "", nextUrl);
    } else {
      window.history.pushState(null, "", nextUrl);
    }
  }

  // --- Data Fetching ---
  async function fetchJson(path) {
    const res = await fetch(path, { cache: "no-cache" });
    if (!res.ok) throw new Error("HTTP " + res.status + " loading " + path);
    return res.json();
  }

  async function loadIssueByDate(dateStr) {
    try {
      const issue = await fetchJson("data/issues/" + dateStr + ".json");
      state.issue = issue;
      renderAll();
    } catch (err) {
      console.error("Failed to load issue:", err);
    }
  }

  async function initData() {
    readUrlParams();
    const params = new URLSearchParams(window.location.search);
    const requestedIssue = params.get("issue");

    try {
      const [archiveRes, tagsRes] = await Promise.allSettled([
        fetchJson("data/archive.json"),
        fetchJson("data/tags.json")
      ]);

      if (archiveRes.status === "fulfilled" && Array.isArray(archiveRes.value)) {
        state.archive = archiveRes.value;
      }
      if (tagsRes.status === "fulfilled" && Array.isArray(tagsRes.value)) {
        state.canonicalTags = tagsRes.value;
      }

      if (requestedIssue) {
        state.issue = await fetchJson("data/issues/" + requestedIssue + ".json");
      } else {
        state.issue = await fetchJson("data/latest.json");
      }
    } catch (err) {
      console.error("Error loading initial Ahbar data:", err);
      const statsEl = document.getElementById("issue-stats");
      if (statsEl) {
        statsEl.innerHTML = "<span>Unable to load issue data. Run <code>cargo run -p ahbar-engine</code> to build data/latest.json.</span>";
      }
      return;
    }

    populateIssueNavigator();
    renderAll();
  }

  // --- Issue Navigator & Banner ---
  function populateIssueNavigator() {
    const select = document.getElementById("issue-select");
    const prevBtn = document.getElementById("prev-issue-btn");
    const nextBtn = document.getElementById("next-issue-btn");
    if (!select || !state.issue) return;

    const entries = state.archive.length > 0
      ? state.archive
      : [{
          issue_number: state.issue.issue_number,
          date: state.issue.date,
          edition_type: state.issue.edition_type,
          qualified_items: state.issue.items ? state.issue.items.length : 0
        }];

    select.innerHTML = entries
      .map((entry) => {
        const badge = entry.edition_type === "weekend" ? "Weekend" : "Issue #" + entry.issue_number;
        const selected = entry.date === state.issue.date ? " selected" : "";
        return `<option value="${escapeHtml(entry.date)}"${selected}>${escapeHtml(badge)} · ${escapeHtml(entry.date)}</option>`;
      })
      .join("");

    updateIssueNavButtons();

    select.onchange = () => {
      state.activeItemId = null;
      loadIssueByDate(select.value);
      pushUrlState(false);
    };

    if (prevBtn) {
      prevBtn.onclick = () => {
        const idx = entries.findIndex((e) => e.date === state.issue.date);
        if (idx >= 0 && idx < entries.length - 1) {
          state.activeItemId = null;
          loadIssueByDate(entries[idx + 1].date);
          pushUrlState(false);
        }
      };
    }

    if (nextBtn) {
      nextBtn.onclick = () => {
        const idx = entries.findIndex((e) => e.date === state.issue.date);
        if (idx > 0) {
          state.activeItemId = null;
          loadIssueByDate(entries[idx - 1].date);
          pushUrlState(false);
        }
      };
    }
  }

  function updateIssueNavButtons() {
    const select = document.getElementById("issue-select");
    const prevBtn = document.getElementById("prev-issue-btn");
    const nextBtn = document.getElementById("next-issue-btn");
    if (!select || !state.issue) return;

    select.value = state.issue.date;
    const entries = state.archive;
    const idx = entries.findIndex((e) => e.date === state.issue.date);
    if (prevBtn) prevBtn.disabled = idx === -1 || idx >= entries.length - 1;
    if (nextBtn) nextBtn.disabled = idx <= 0;
  }

  function renderBanner() {
    if (!state.issue) return;
    const headingEl = document.getElementById("edition-heading");
    const statsEl = document.getElementById("issue-stats");
    const isWeekend = state.issue.edition_type === "weekend";

    if (headingEl) {
      if (isWeekend) {
        headingEl.innerHTML = `Weekend Edition <em>#${state.issue.issue_number}</em> — Deep-Dive &amp; Synthesis`;
      } else {
        headingEl.innerHTML = `Daily Issue <em>#${state.issue.issue_number}</em> — Technical &amp; Scientific Dispatches`;
      }
    }

    if (statsEl && state.issue.stats) {
      const s = state.issue.stats;
      statsEl.innerHTML = `
        <span><strong>${escapeHtml(state.issue.date)}</strong></span>
        <span>·</span>
        <span><strong>${s.qualified_items}</strong> qualified dispatches</span>
        <span>·</span>
        <span><strong>${s.raw_items_scanned}</strong> raw papers &amp; feeds triaged</span>
        <span>·</span>
        <span><strong>${s.sources_healthy}/${s.sources_checked}</strong> sources online</span>
      `;
    }
  }

  // --- Filtering & Sorting Logic ---
  function matchesItem(item, ignoreTag) {
    if (state.area !== "all" && item.area !== state.area) return false;
    if (!ignoreTag && state.tag && !(item.tags || []).includes(state.tag)) return false;

    if (state.filters.code && !(item.links && item.links.code_url)) return false;
    if (state.filters.papers && !item.is_paper) return false;
    if (
      state.filters.discussed &&
      !(
        (item.links && item.links.discussion_url) ||
        (item.metrics && item.metrics.community_points > 0)
      )
    ) {
      return false;
    }
    if (state.filters.unread && state.readMap[item.id]) return false;

    if (state.query) {
      const q = state.query.toLowerCase().trim();
      const cleanTag = q.startsWith("#") ? q.slice(1) : q;
      const inTags = (item.tags || []).some((t) => t.toLowerCase().includes(cleanTag));
      if (q.startsWith("#")) return inTags;

      const haystack = [
        item.headline,
        item.summary,
        item.original_title,
        item.source_name,
        (item.authors || []).join(" "),
        (item.tags || []).join(" ")
      ]
        .join(" ")
        .toLowerCase();
      if (!haystack.includes(q)) return false;
    }

    return true;
  }

  function getFilteredItems() {
    if (!state.issue || !Array.isArray(state.issue.items)) return [];
    const filtered = state.issue.items.filter((item) => matchesItem(item, false));

    filtered.sort((a, b) => {
      if (state.sort === "community") {
        const pa = (a.metrics && a.metrics.community_points) || 0;
        const pb = (b.metrics && b.metrics.community_points) || 0;
        if (pb !== pa) return pb - pa;
      } else if (state.sort === "source") {
        const cmp = (a.source_name || "").localeCompare(b.source_name || "");
        if (cmp !== 0) return cmp;
      }
      const sa = (a.metrics && a.metrics.final_score) || 0;
      const sb = (b.metrics && b.metrics.final_score) || 0;
      return sb - sa;
    });

    return filtered;
  }

  // --- Rendering Top Picks Showcase ---
  function renderTopPicks() {
    const section = document.getElementById("top-picks-section");
    const grid = document.getElementById("top-picks-grid");
    if (!section || !grid || !state.issue) return;

    // Hide top picks showcase when searching or filtering by specific tag
    const hasNarrowFilter =
      state.area !== "all" ||
      state.tag !== null ||
      state.query.trim() !== "" ||
      state.filters.code ||
      state.filters.papers ||
      state.filters.discussed ||
      state.filters.unread;

    const topPicks = (state.issue.items || []).filter((i) => i.is_top_pick);
    if (hasNarrowFilter || topPicks.length === 0) {
      section.hidden = true;
      return;
    }

    section.hidden = false;
    grid.innerHTML = topPicks
      .slice(0, 6)
      .map((item, idx) => {
        const areaMeta = getAreaMeta(item.area);
        const isRead = !!state.readMap[item.id];
        const isSaved = !!state.savedMap[item.id];
        const tagsHtml = (item.tags || [])
          .slice(0, 3)
          .map(
            (t) =>
              `<button type="button" class="chip tag-trigger" data-tag="${escapeHtml(t)}" style="padding:2px 8px;font-size:11.5px;">#${escapeHtml(t)}</button>`
          )
          .join("");

        return `
          <article class="pick-card${isRead ? " is-read" : ""}" data-item-id="${escapeHtml(item.id)}">
            <div>
              <div class="item-meta-top">
                <span class="item-seq">0${idx + 1}</span>
                <span class="item-area-badge">${escapeHtml(areaMeta.numeral)} · ${escapeHtml(areaMeta.short)}</span>
                <span>·</span>
                <span>${escapeHtml(item.source_name)}</span>
                <span class="score-badge" title="Composite Technical & Multi-Signal Score">${item.metrics.final_score.toFixed(1)}</span>
              </div>
              <a href="?item=${encodeURIComponent(item.id)}" class="item-headline item-detail-link" data-item-id="${escapeHtml(item.id)}" style="font-size:16.5px;">
                ${escapeHtml(item.headline)}
              </a>
              <p class="item-summary" style="font-size:14px;margin-bottom:10px;">
                ${escapeHtml(item.summary)}
              </p>
            </div>
            <div class="item-footer" style="padding-top:8px;border-top:1px solid var(--rule2);">
              <div class="item-badges">
                ${tagsHtml}
              </div>
              <div class="item-actions">
                <a href="?item=${encodeURIComponent(item.id)}" class="badge-link detail-cta item-detail-link" data-item-id="${escapeHtml(item.id)}">Analysis ↗</a>
                <button type="button" class="act-btn save-trigger${isSaved ? " active" : ""}" data-item-id="${escapeHtml(item.id)}" title="Save to reading list">
                  ${isSaved ? "★ Saved" : "☆ Save"}
                </button>
              </div>
            </div>
          </article>
        `;
      })
      .join("");
  }

  // --- Rendering Area Tabs & Tag Chips ---
  function renderControls() {
    if (!state.issue) return;
    const allItems = state.issue.items || [];

    // 1. Area Tabs
    const areaTabsEl = document.getElementById("area-tabs");
    if (areaTabsEl) {
      areaTabsEl.innerHTML = AREAS.map((area) => {
        const count =
          area.slug === "all"
            ? allItems.length
            : allItems.filter((i) => i.area === area.slug).length;
        const active = state.area === area.slug;
        const label =
          area.slug === "all"
            ? `All Areas`
            : `<span class="rn">${escapeHtml(area.numeral)}</span> ${escapeHtml(area.short)}`;
        return `
          <button type="button" class="chip${active ? " on" : ""}" role="tab" aria-selected="${active}" data-area="${escapeHtml(area.slug)}">
            ${label}
            <span class="c">${count}</span>
          </button>
        `;
      }).join("");
    }

    // 2. Quick Technical Filters
    const qfContainer = document.getElementById("quick-filters");
    if (qfContainer) {
      qfContainer.querySelectorAll("[data-filter]").forEach((btn) => {
        const key = btn.getAttribute("data-filter");
        const on = !!state.filters[key];
        btn.setAttribute("aria-pressed", on ? "true" : "false");
        btn.classList.toggle("on", on);
      });
    }

    // 3. Canonical Tag Bar (counts computed against current area & quick filters)
    const tagBarEl = document.getElementById("tag-bar");
    if (tagBarEl) {
      const tagCounts = {};
      allItems.forEach((item) => {
        if (matchesItem(item, true)) {
          (item.tags || []).forEach((t) => {
            tagCounts[t] = (tagCounts[t] || 0) + 1;
          });
        }
      });

      const sortedTags = Object.keys(tagCounts).sort((a, b) => {
        if (tagCounts[b] !== tagCounts[a]) return tagCounts[b] - tagCounts[a];
        return a.localeCompare(b);
      });

      if (sortedTags.length === 0) {
        tagBarEl.innerHTML = `<span class="n">No canonical tags match the current filter.</span>`;
      } else {
        const resetBtn = state.tag
          ? `<button type="button" class="chip on" data-tag="">✕ Clear #${escapeHtml(state.tag)}</button>`
          : "";
        tagBarEl.innerHTML =
          resetBtn +
          sortedTags
            .map((t) => {
              const active = state.tag === t;
              return `
                <button type="button" class="chip${active ? " on" : ""}" data-tag="${escapeHtml(t)}">
                  #${escapeHtml(t)}
                  <span class="c">${tagCounts[t]}</span>
                </button>
              `;
            })
            .join("");
      }
    }

    // 4. Update RSS / Atom Feed Link dynamically based on selection
    const rssLink = document.getElementById("current-rss-link");
    if (rssLink) {
      if (state.tag) {
        rssLink.href = `feed/tag-${encodeURIComponent(state.tag)}.xml`;
        rssLink.textContent = `Atom: #${state.tag} ↗`;
      } else if (state.area !== "all") {
        rssLink.href = `feed/area-${encodeURIComponent(state.area)}.xml`;
        rssLink.textContent = `Atom: ${getAreaMeta(state.area).short} ↗`;
      } else {
        rssLink.href = `feed/all.xml`;
        rssLink.textContent = `RSS / Atom ↗`;
      }
    }
  }

  // --- Rendering Main Feed List ---
  function renderFeedList() {
    const listEl = document.getElementById("feed-list");
    const summaryEl = document.getElementById("active-filter-summary");
    if (!listEl) return;

    const items = getFilteredItems();
    state.visibleItems = items;

    if (summaryEl) {
      const areaLabel = getAreaMeta(state.area).title;
      const tagLabel = state.tag ? ` · #${state.tag}` : "";
      summaryEl.textContent = `${items.length} Dispatches · ${areaLabel}${tagLabel}`;
    }

    if (items.length === 0) {
      listEl.innerHTML = `
        <div style="padding:36px 20px;text-align:center;color:var(--dim);background:var(--field);border:1px solid var(--rule);border-radius:8px;">
          <p style="margin:0 0 10px;font-weight:600;">No technical dispatches match the active filters.</p>
          <button type="button" class="btn" id="reset-filters-btn">Reset All Filters</button>
        </div>
      `;
      const resetBtn = document.getElementById("reset-filters-btn");
      if (resetBtn) {
        resetBtn.onclick = () => {
          state.area = "all";
          state.tag = null;
          state.query = "";
          state.filters = { code: false, papers: false, discussed: false, unread: false };
          const qInput = document.getElementById("q");
          if (qInput) qInput.value = "";
          pushUrlState(false);
          renderAll();
        };
      }
      return;
    }

    listEl.innerHTML = items
      .map((item, idx) => {
        const areaMeta = getAreaMeta(item.area);
        const isRead = !!state.readMap[item.id];
        const isSaved = !!state.savedMap[item.id];
        const vote = state.voteMap[item.id] || null;
        const seqNum = String(idx + 1).padStart(2, "0");

        const links = item.links || {};
        const badges = [];
        badges.push(
          `<a href="?item=${encodeURIComponent(item.id)}" class="badge-link detail-cta item-detail-link" data-item-id="${escapeHtml(item.id)}">[Analysis &amp; Cite]</a>`
        );
        badges.push(
          `<a href="${escapeHtml(item.url)}" target="_blank" rel="noopener" class="badge-link">[Source ↗]</a>`
        );
        if (links.pdf_url) {
          badges.push(
            `<a href="${escapeHtml(links.pdf_url)}" target="_blank" rel="noopener" class="badge-link">[PDF]</a>`
          );
        }
        if (links.html_ar5iv_url) {
          badges.push(
            `<a href="${escapeHtml(links.html_ar5iv_url)}" target="_blank" rel="noopener" class="badge-link">[HTML / ar5iv]</a>`
          );
        }
        if (links.code_url) {
          badges.push(
            `<a href="${escapeHtml(links.code_url)}" target="_blank" rel="noopener" class="badge-link">[Code]</a>`
          );
        }
        if (links.discussion_url) {
          const dLabel = links.discussion_label || "Discussion";
          badges.push(
            `<a href="${escapeHtml(links.discussion_url)}" target="_blank" rel="noopener" class="badge-link">[${escapeHtml(dLabel)}]</a>`
          );
        }

        const tagChips = (item.tags || [])
          .map(
            (t) =>
              `<button type="button" class="chip tag-trigger${state.tag === t ? " on" : ""}" data-tag="${escapeHtml(t)}" style="padding:2px 9px;font-size:11.5px;">#${escapeHtml(t)}</button>`
          )
          .join("");

        const authorsStr =
          item.authors && item.authors.length > 0
            ? ` · <span>${escapeHtml(item.authors.slice(0, 3).join(", "))}${item.authors.length > 3 ? " et al." : ""}</span>`
            : "";

        const multiSourceStr =
          item.metrics && item.metrics.cross_source_count > 1
            ? ` · <span title="Clustered across multiple technical feeds">${item.metrics.cross_source_count} sources</span>`
            : "";

        return `
          <article class="news-row${isRead ? " is-read" : ""}${state.kbIndex === idx ? " kb-focus" : ""}" data-item-id="${escapeHtml(item.id)}" data-idx="${idx}">
            <div class="item-meta-top">
              <span class="item-seq">${seqNum}</span>
              <span class="item-area-badge">${escapeHtml(areaMeta.numeral)} · ${escapeHtml(areaMeta.title)}</span>
              <span>·</span>
              <strong>${escapeHtml(item.source_name)}</strong>
              ${authorsStr}
              ${multiSourceStr}
              <span class="score-badge" title="Base ${item.metrics.base_score.toFixed(1)} | Novelty ${item.metrics.llm_novelty.toFixed(1)} | Rigor ${item.metrics.llm_rigor.toFixed(1)}">
                Score ${item.metrics.final_score.toFixed(1)}
              </span>
            </div>

            <a href="?item=${encodeURIComponent(item.id)}" class="item-headline item-detail-link" data-item-id="${escapeHtml(item.id)}">
              ${escapeHtml(item.headline)}
            </a>

            <p class="item-summary">${escapeHtml(item.summary)}</p>

            <div class="item-footer">
              <div class="item-badges">
                ${badges.join("")}
                ${tagChips}
              </div>

              <div class="item-actions">
                <button type="button" class="act-btn vote-trigger${vote === "signal" ? " active" : ""}" data-item-id="${escapeHtml(item.id)}" data-vote="signal" title="High technical signal">
                  ▲ Signal
                </button>
                <button type="button" class="act-btn vote-trigger${vote === "noise" ? " active" : ""}" data-item-id="${escapeHtml(item.id)}" data-vote="noise" title="Low signal / Noise">
                  ▼ Noise
                </button>
                <button type="button" class="act-btn save-trigger${isSaved ? " active" : ""}" data-item-id="${escapeHtml(item.id)}" title="Save to reading list (s)">
                  ${isSaved ? "★ Saved" : "☆ Save"}
                </button>
              </div>
            </div>
          </article>
        `;
      })
      .join("");
  }

  // --- Layer-2 Item Detail Page (?item=<id>) ---
  function renderDetailView(item) {
    const detailEl = document.getElementById("detail-view");
    if (!detailEl) return;

    // Mark item as read
    state.readMap[item.id] = Date.now();
    saveJson(STORAGE_KEYS.read, state.readMap);

    const areaMeta = getAreaMeta(item.area);
    const isSaved = !!state.savedMap[item.id];
    const vote = state.voteMap[item.id] || null;
    const links = item.links || {};
    const detail = item.detail || {};

    const actionButtons = [];
    actionButtons.push(
      `<a href="${escapeHtml(item.url)}" target="_blank" rel="noopener" class="btn primary">Open Primary Source ↗</a>`
    );
    if (links.pdf_url) {
      actionButtons.push(
        `<a href="${escapeHtml(links.pdf_url)}" target="_blank" rel="noopener" class="btn">Read PDF ↗</a>`
      );
    }
    if (links.html_ar5iv_url) {
      actionButtons.push(
        `<a href="${escapeHtml(links.html_ar5iv_url)}" target="_blank" rel="noopener" class="btn">Read HTML (ar5iv) ↗</a>`
      );
    }
    if (links.code_url) {
      actionButtons.push(
        `<a href="${escapeHtml(links.code_url)}" target="_blank" rel="noopener" class="btn">Repository / Code ↗</a>`
      );
    }
    if (links.discussion_url) {
      actionButtons.push(
        `<a href="${escapeHtml(links.discussion_url)}" target="_blank" rel="noopener" class="btn">${escapeHtml(links.discussion_label || "Discussion")} ↗</a>`
      );
    }

    const clusteredList = (links.clustered_sources || [])
      .map(
        (cs) => `
        <li>
          <div>
            <strong>${escapeHtml(cs.source_name)}:</strong>
            <a href="${escapeHtml(cs.url)}" target="_blank" rel="noopener">${escapeHtml(cs.title)} ↗</a>
          </div>
          ${cs.points ? `<span class="n">${cs.points} pts</span>` : ""}
        </li>
      `
      )
      .join("");

    // Related items from same issue
    const related = (state.issue.items || [])
      .filter(
        (other) =>
          other.id !== item.id &&
          (other.area === item.area ||
            (other.tags || []).some((t) => (item.tags || []).includes(t)))
      )
      .slice(0, 3);

    const relatedHtml =
      related.length > 0
        ? `
        <div class="sources-box">
          <h3 class="k">Related Technical Dispatches</h3>
          <ul class="sources-list">
            ${related
              .map(
                (r) => `
              <li>
                <a href="?item=${encodeURIComponent(r.id)}" class="item-detail-link" data-item-id="${escapeHtml(r.id)}" style="font-weight:600;text-decoration:none;">
                  ${escapeHtml(r.headline)}
                </a>
                <span class="n">${escapeHtml(r.source_name)}</span>
              </li>
            `
              )
              .join("")}
          </ul>
        </div>
      `
        : "";

    const bibtexBlock = detail.bibtex
      ? `
        <div class="sources-box">
          <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:8px;">
            <h3 class="k" style="margin:0;">BibTeX Citation</h3>
            <button type="button" class="btn" id="copy-bibtex-btn">Copy BibTeX</button>
          </div>
          <pre class="bibtex-pre" id="bibtex-content">${escapeHtml(detail.bibtex)}</pre>
        </div>
      `
      : "";

    const abstractBlock = detail.abstract_excerpt
      ? `
        <details class="analysis-block" style="margin-bottom:20px;">
          <summary style="cursor:pointer;font-weight:600;color:var(--dim);">
            Original Paper Abstract / Excerpt
          </summary>
          <p style="margin-top:10px;">${escapeHtml(detail.abstract_excerpt)}</p>
        </details>
      `
      : "";

    detailEl.innerHTML = `
      <div class="detail-back-bar">
        <button type="button" class="btn" id="back-to-feed-btn">← Back to Issue #${state.issue.issue_number} Feed</button>
        <div style="display:flex;gap:8px;">
          <button type="button" class="btn" id="copy-md-btn">Copy Markdown</button>
          <button type="button" class="btn save-trigger${isSaved ? " primary" : ""}" data-item-id="${escapeHtml(item.id)}">
            ${isSaved ? "★ Saved in Reading List" : "☆ Save Dispatch"}
          </button>
        </div>
      </div>

      <div class="item-meta-top">
        <span class="item-area-badge">${escapeHtml(areaMeta.numeral)} · ${escapeHtml(areaMeta.title)}</span>
        <span>·</span>
        <strong>${escapeHtml(item.source_name)}</strong>
        <span>·</span>
        <span>Issue #${item.issue_number} (${escapeHtml(item.issue_date)})</span>
        <span class="score-badge">Score ${item.metrics.final_score.toFixed(1)} (Novelty ${item.metrics.llm_novelty.toFixed(1)} · Rigor ${item.metrics.llm_rigor.toFixed(1)})</span>
      </div>

      <h1 class="detail-headline">${escapeHtml(item.headline)}</h1>

      <div class="detail-orig-title">
        <strong>Original Title:</strong> ${escapeHtml(item.original_title)}
        ${
          item.authors && item.authors.length > 0
            ? `<br><strong>Authors:</strong> ${escapeHtml(item.authors.join(", "))}`
            : ""
        }
      </div>

      <div class="detail-toolbar">
        ${actionButtons.join("")}
        <div style="margin-left:auto;display:flex;gap:6px;">
          <button type="button" class="act-btn vote-trigger${vote === "signal" ? " active" : ""}" data-item-id="${escapeHtml(item.id)}" data-vote="signal">▲ High Signal</button>
          <button type="button" class="act-btn vote-trigger${vote === "noise" ? " active" : ""}" data-item-id="${escapeHtml(item.id)}" data-vote="noise">▼ Noise</button>
        </div>
      </div>

      <div class="detail-lead">
        ${escapeHtml(item.summary)}
      </div>

      <div class="analysis-grid">
        <div class="analysis-block">
          <h3>I. Problem &amp; Technical Context</h3>
          <p>${escapeHtml(detail.problem_context || item.summary)}</p>
        </div>
        <div class="analysis-block">
          <h3>II. Method, Architecture &amp; Formal Construction</h3>
          <p>${escapeHtml(detail.method_architecture || "See primary source for full methodological specification.")}</p>
        </div>
        <div class="analysis-block">
          <h3>III. Empirical Findings, Benchmarks &amp; Trade-offs</h3>
          <p>${escapeHtml(detail.findings_tradeoffs || "Refer to the paper or repository for complete benchmark tables and limitations.")}</p>
        </div>
      </div>

      ${abstractBlock}

      <div style="display:flex;flex-wrap:wrap;gap:6px;margin-bottom:20px;">
        ${(item.tags || [])
          .map(
            (t) =>
              `<button type="button" class="chip tag-trigger" data-tag="${escapeHtml(t)}">#${escapeHtml(t)}</button>`
          )
          .join("")}
      </div>

      ${
        clusteredList
          ? `
        <div class="sources-box">
          <h3 class="k">Clustered Sources &amp; Discussions (${links.clustered_sources.length})</h3>
          <ul class="sources-list">${clusteredList}</ul>
        </div>
      `
          : ""
      }

      ${bibtexBlock}
      ${relatedHtml}
    `;

    const backBtn = document.getElementById("back-to-feed-btn");
    if (backBtn) {
      backBtn.onclick = () => {
        state.activeItemId = null;
        pushUrlState(false);
        renderAll();
        window.scrollTo({ top: 0, behavior: "smooth" });
      };
    }

    const copyMdBtn = document.getElementById("copy-md-btn");
    if (copyMdBtn) {
      copyMdBtn.onclick = () => {
        const md = formatItemMarkdown(item);
        navigator.clipboard.writeText(md).then(() => {
          copyMdBtn.textContent = "✓ Copied Markdown";
          setTimeout(() => (copyMdBtn.textContent = "Copy Markdown"), 1800);
        });
      };
    }

    const copyBibBtn = document.getElementById("copy-bibtex-btn");
    if (copyBibBtn && detail.bibtex) {
      copyBibBtn.onclick = () => {
        navigator.clipboard.writeText(detail.bibtex).then(() => {
          copyBibBtn.textContent = "✓ Copied BibTeX";
          setTimeout(() => (copyBibBtn.textContent = "Copy BibTeX"), 1800);
        });
      };
    }
  }

  function formatItemMarkdown(item) {
    const tags = (item.tags || []).map((t) => `#${t}`).join(" ");
    return [
      `### ${item.headline}`,
      `- **Original Title:** ${item.original_title}`,
      `- **Source:** [${item.source_name}](${item.url}) (${item.issue_date})`,
      item.links && item.links.pdf_url ? `- **PDF:** ${item.links.pdf_url}` : null,
      item.links && item.links.code_url ? `- **Code:** ${item.links.code_url}` : null,
      `- **Tags:** ${tags}`,
      "",
      item.summary,
      ""
    ]
      .filter((line) => line !== null)
      .join("\n");
  }

  // --- Saved Reading List & Export ---
  function updateSavedBadge() {
    const countEl = document.getElementById("saved-count");
    if (countEl) {
      countEl.textContent = String(Object.keys(state.savedMap).length);
    }
  }

  function toggleSaveItem(itemId) {
    if (!state.issue) return;
    if (state.savedMap[itemId]) {
      delete state.savedMap[itemId];
    } else {
      const found = (state.issue.items || []).find((i) => i.id === itemId);
      if (found) {
        state.savedMap[itemId] = found;
      }
    }
    saveJson(STORAGE_KEYS.saved, state.savedMap);
    updateSavedBadge();
    renderAll();
  }

  function recordVote(itemId, voteType) {
    if (state.voteMap[itemId] === voteType) {
      delete state.voteMap[itemId];
    } else {
      state.voteMap[itemId] = voteType;
    }
    saveJson(STORAGE_KEYS.votes, state.voteMap);

    // Optional edge telemetry if API endpoint is configured
    if (window.AHBAR_API_BASE && state.voteMap[itemId]) {
      fetch(window.AHBAR_API_BASE + "/api/vote", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ item_id: itemId, vote: state.voteMap[itemId] })
      }).catch(() => {});
    }
    renderAll();
  }

  function renderSavedModal() {
    const container = document.getElementById("saved-list-container");
    if (!container) return;
    const savedItems = Object.values(state.savedMap);

    if (savedItems.length === 0) {
      container.innerHTML = `<p style="color:var(--dim);font-size:14px;">Your local reading list is empty. Click <strong>☆ Save</strong> or press <kbd>s</kbd> on any dispatch to bookmark it for Markdown or BibTeX export.</p>`;
      return;
    }

    container.innerHTML = savedItems
      .map(
        (item) => `
        <div style="padding:12px 0;border-bottom:1px solid var(--rule2);display:flex;align-items:flex-start;justify-content:space-between;gap:12px;">
          <div>
            <div class="n" style="margin-bottom:4px;">${escapeHtml(item.source_name)} · ${escapeHtml(item.issue_date)}</div>
            <a href="?item=${encodeURIComponent(item.id)}" class="item-detail-link" data-item-id="${escapeHtml(item.id)}" style="font-weight:600;text-decoration:none;">
              ${escapeHtml(item.headline)}
            </a>
          </div>
          <button type="button" class="btn save-trigger" data-item-id="${escapeHtml(item.id)}" style="padding:4px 9px;font-size:12px;">Remove</button>
        </div>
      `
      )
      .join("");
  }

  function downloadFile(filename, content, mime) {
    const blob = new Blob([content], { type: mime || "text/plain;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  // --- Subscription & Preferences Modal ---
  function openSubModal() {
    const modal = document.getElementById("sub-modal");
    if (!modal) return;
    modal.hidden = false;

    const emailInput = document.getElementById("sub-email");
    const freqSelect = document.getElementById("sub-frequency");
    const filterSelect = document.getElementById("sub-content-filter");
    if (emailInput) emailInput.value = state.subPrefs.email || "";
    if (freqSelect) freqSelect.value = state.subPrefs.frequency || "daily";
    if (filterSelect) filterSelect.value = state.subPrefs.content_filter || "all";

    // Sync radio cards
    document.querySelectorAll("#mode-options .radio-card").forEach((card) => {
      const val = card.getAttribute("data-val");
      const active = val === (state.subPrefs.mode || "hybrid");
      card.classList.toggle("selected", active);
      const input = card.querySelector("input");
      if (input) input.checked = active;
    });

    document.querySelectorAll("#density-options .radio-card").forEach((card) => {
      const val = card.getAttribute("data-val");
      const active = val === (state.subPrefs.density || "compact");
      card.classList.toggle("selected", active);
      const input = card.querySelector("input");
      if (input) input.checked = active;
    });

    renderSubPickers();
  }

  function renderSubPickers() {
    const areasPicker = document.getElementById("sub-areas-picker");
    const tagsPicker = document.getElementById("sub-tags-picker");

    if (areasPicker) {
      areasPicker.innerHTML = AREAS.filter((a) => a.slug !== "all")
        .map((a) => {
          const on = (state.subPrefs.areas || []).includes(a.slug);
          return `
            <button type="button" class="chip sub-area-chip${on ? " on" : ""}" data-area="${escapeHtml(a.slug)}">
              <span class="rn">${escapeHtml(a.numeral)}</span> ${escapeHtml(a.short)}
            </button>
          `;
        })
        .join("");
    }

    if (tagsPicker) {
      const allTags =
        state.canonicalTags.length > 0
          ? state.canonicalTags.map((t) => t.slug)
          : [
              "llm",
              "agents",
              "ml-theory",
              "pl-compilers",
              "distributed-systems",
              "databases",
              "rust",
              "formal-verification",
              "number-theory",
              "combinatorics",
              "quantum",
              "condensed-matter",
              "os-kernels",
              "hardware-arch",
              "infosec",
              "bioinformatics",
              "zk-proofs",
              "cryptography",
              "consensus"
            ];

      tagsPicker.innerHTML = allTags
        .map((slug) => {
          const on = (state.subPrefs.tags || []).includes(slug);
          return `
            <button type="button" class="chip sub-tag-chip${on ? " on" : ""}" data-tag="${escapeHtml(slug)}" style="padding:3px 9px;font-size:11.5px;">
              #${escapeHtml(slug)}
            </button>
          `;
        })
        .join("");
    }
  }

  // --- Main Render Coordinator ---
  function renderAll() {
    if (!state.issue) return;
    renderBanner();
    updateIssueNavButtons();
    updateSavedBadge();

    const feedView = document.getElementById("feed-view");
    const detailView = document.getElementById("detail-view");

    if (state.activeItemId) {
      const item = (state.issue.items || []).find((i) => i.id === state.activeItemId);
      if (item && feedView && detailView) {
        feedView.hidden = true;
        detailView.hidden = false;
        renderDetailView(item);
        triggerMathRender(detailView);
        return;
      } else {
        state.activeItemId = null;
      }
    }

    if (feedView && detailView) {
      feedView.hidden = false;
      detailView.hidden = true;
    }

    renderTopPicks();
    renderControls();
    renderFeedList();
    triggerMathRender(feedView);
  }

  // --- Event Listeners & Keyboard Shortcuts ---
  function bindEvents() {
    // Home Logo resets filters and returns to feed view
    const homeLogo = document.getElementById("home-logo");
    if (homeLogo) {
      homeLogo.addEventListener("click", (e) => {
        e.preventDefault();
        state.activeItemId = null;
        state.area = "all";
        state.tag = null;
        state.query = "";
        const qInput = document.getElementById("q");
        if (qInput) qInput.value = "";
        pushUrlState(false);
        renderAll();
      });
    }

    // Search Box
    const qInput = document.getElementById("q");
    if (qInput) {
      qInput.addEventListener("input", () => {
        state.query = qInput.value;
        state.activeItemId = null;
        state.kbIndex = -1;
        pushUrlState(true);
        renderAll();
      });
    }

    // Sort Select
    const sortSelect = document.getElementById("sort-select");
    if (sortSelect) {
      sortSelect.addEventListener("change", () => {
        state.sort = sortSelect.value;
        renderAll();
      });
    }

    // Delegated Clicks across Document
    document.addEventListener("click", (e) => {
      // 1. Area Tab
      const areaBtn = e.target.closest("#area-tabs [data-area]");
      if (areaBtn) {
        state.area = areaBtn.getAttribute("data-area") || "all";
        state.tag = null;
        state.kbIndex = -1;
        pushUrlState(false);
        renderAll();
        return;
      }

      // 2. Quick Filter
      const filterBtn = e.target.closest("#quick-filters [data-filter]");
      if (filterBtn) {
        const key = filterBtn.getAttribute("data-filter");
        state.filters[key] = !state.filters[key];
        state.kbIndex = -1;
        renderAll();
        return;
      }

      // 3. Tag Trigger (in tag bar or on item card)
      const tagBtn = e.target.closest("#tag-bar [data-tag], .tag-trigger");
      if (tagBtn) {
        const clickedTag = tagBtn.getAttribute("data-tag");
        state.tag = !clickedTag || state.tag === clickedTag ? null : clickedTag;
        state.activeItemId = null;
        state.kbIndex = -1;
        pushUrlState(false);
        renderAll();
        return;
      }

      // 4. Item Detail Link (?item=<id>)
      const detailLink = e.target.closest(".item-detail-link");
      if (detailLink && !e.ctrlKey && !e.metaKey && !e.shiftKey) {
        e.preventDefault();
        const itemId = detailLink.getAttribute("data-item-id");
        if (itemId) {
          state.activeItemId = itemId;
          pushUrlState(false);
          const savedModal = document.getElementById("saved-modal");
          if (savedModal) savedModal.hidden = true;
          renderAll();
          window.scrollTo({ top: 0, behavior: "smooth" });
        }
        return;
      }

      // 5. Save Button
      const saveBtn = e.target.closest(".save-trigger");
      if (saveBtn) {
        const itemId = saveBtn.getAttribute("data-item-id");
        if (itemId) {
          toggleSaveItem(itemId);
          renderSavedModal();
        }
        return;
      }

      // 6. Vote Button
      const voteBtn = e.target.closest(".vote-trigger");
      if (voteBtn) {
        const itemId = voteBtn.getAttribute("data-item-id");
        const voteType = voteBtn.getAttribute("data-vote");
        if (itemId && voteType) {
          recordVote(itemId, voteType);
        }
        return;
      }

      // 7. Subscription Modal Radio Cards & Chips
      const radioCard = e.target.closest(".radio-card");
      if (radioCard) {
        const group = radioCard.parentElement;
        if (group) {
          group.querySelectorAll(".radio-card").forEach((c) => c.classList.remove("selected"));
          radioCard.classList.add("selected");
          const input = radioCard.querySelector("input");
          if (input) {
            input.checked = true;
            if (input.name === "mode") state.subPrefs.mode = input.value;
            if (input.name === "density") state.subPrefs.density = input.value;
          }
        }
        return;
      }

      const subAreaChip = e.target.closest(".sub-area-chip");
      if (subAreaChip) {
        const slug = subAreaChip.getAttribute("data-area");
        const list = state.subPrefs.areas || [];
        const idx = list.indexOf(slug);
        if (idx >= 0) list.splice(idx, 1);
        else list.push(slug);
        state.subPrefs.areas = list;
        renderSubPickers();
        return;
      }

      const subTagChip = e.target.closest(".sub-tag-chip");
      if (subTagChip) {
        const slug = subTagChip.getAttribute("data-tag");
        const list = state.subPrefs.tags || [];
        const idx = list.indexOf(slug);
        if (idx >= 0) list.splice(idx, 1);
        else list.push(slug);
        state.subPrefs.tags = list;
        renderSubPickers();
        return;
      }
    });

    // Modals Open / Close
    const subBtn = document.getElementById("subscribe-btn");
    const subModal = document.getElementById("sub-modal");
    const closeSubBtn = document.getElementById("close-sub-modal");
    if (subBtn) subBtn.onclick = () => openSubModal();
    if (closeSubBtn && subModal) closeSubBtn.onclick = () => (subModal.hidden = true);

    const savedBtn = document.getElementById("saved-btn");
    const savedModal = document.getElementById("saved-modal");
    const closeSavedBtn = document.getElementById("close-saved-modal");
    if (savedBtn && savedModal) {
      savedBtn.onclick = () => {
        renderSavedModal();
        savedModal.hidden = false;
      };
    }
    if (closeSavedBtn && savedModal) closeSavedBtn.onclick = () => (savedModal.hidden = true);

    // Saved Modal Export Buttons
    const exportMdBtn = document.getElementById("export-md-btn");
    if (exportMdBtn) {
      exportMdBtn.onclick = () => {
        const items = Object.values(state.savedMap);
        if (items.length === 0) return;
        const md =
          `# Ahbar — Saved Technical Dispatches (${new Date().toISOString().slice(0, 10)})\n\n` +
          items.map((item) => formatItemMarkdown(item)).join("\n---\n\n");
        downloadFile("ahbar-saved-dispatches.md", md, "text/markdown;charset=utf-8");
      };
    }

    const exportBibBtn = document.getElementById("export-bib-btn");
    if (exportBibBtn) {
      exportBibBtn.onclick = () => {
        const items = Object.values(state.savedMap);
        if (items.length === 0) return;
        const bibs = items
          .map((item) => (item.detail && item.detail.bibtex) || "")
          .filter(Boolean)
          .join("\n\n");
        downloadFile("ahbar-citations.bib", bibs || "% No BibTeX entries found.\n", "application/x-bibtex;charset=utf-8");
      };
    }

    const clearSavedBtn = document.getElementById("clear-saved-btn");
    if (clearSavedBtn) {
      clearSavedBtn.onclick = () => {
        state.savedMap = {};
        saveJson(STORAGE_KEYS.saved, state.savedMap);
        updateSavedBadge();
        renderSavedModal();
        renderAll();
      };
    }

    // Subscription Form Submit
    const subForm = document.getElementById("sub-form");
    if (subForm) {
      subForm.addEventListener("submit", async (e) => {
        e.preventDefault();
        const emailEl = document.getElementById("sub-email");
        const freqEl = document.getElementById("sub-frequency");
        const filterEl = document.getElementById("sub-content-filter");
        const statusEl = document.getElementById("sub-status-msg");

        state.subPrefs.email = emailEl ? emailEl.value.trim() : "";
        state.subPrefs.frequency = freqEl ? freqEl.value : "daily";
        state.subPrefs.content_filter = filterEl ? filterEl.value : "all";
        saveJson(STORAGE_KEYS.prefs, state.subPrefs);

        if (window.AHBAR_API_BASE) {
          try {
            const res = await fetch(window.AHBAR_API_BASE + "/api/subscribe", {
              method: "POST",
              headers: { "Content-Type": "application/json" },
              body: JSON.stringify(state.subPrefs)
            });
            const data = await res.json();
            if (statusEl) {
              statusEl.textContent = data.message || "Preferences saved & verification link dispatched.";
            }
            return;
          } catch (_) {}
        }

        if (statusEl) {
          statusEl.textContent =
            "✓ Saved locally (" +
            state.subPrefs.mode +
            " · " +
            state.subPrefs.density +
            " · " +
            state.subPrefs.areas.length +
            " areas). Connect Cloudflare Worker or Rust API for live SMTP delivery.";
        }
      });
    }

    // Popstate (Browser Back / Forward)
    window.addEventListener("popstate", () => {
      readUrlParams();
      renderAll();
    });

    // Keyboard Navigation (j/k, Enter, o, s, /, Esc)
    document.addEventListener("keydown", (e) => {
      const activeTag = document.activeElement ? document.activeElement.tagName : "";
      const inInput = activeTag === "INPUT" || activeTag === "TEXTAREA" || activeTag === "SELECT";

      if (e.key === "Escape") {
        if (subModal && !subModal.hidden) {
          subModal.hidden = true;
          return;
        }
        if (savedModal && !savedModal.hidden) {
          savedModal.hidden = true;
          return;
        }
        if (inInput && document.activeElement) {
          document.activeElement.blur();
          return;
        }
        if (state.activeItemId) {
          state.activeItemId = null;
          pushUrlState(false);
          renderAll();
          return;
        }
      }

      if (inInput) return;

      if (e.key === "/") {
        e.preventDefault();
        if (qInput) qInput.focus();
        return;
      }

      if (state.activeItemId) return;

      const len = state.visibleItems.length;
      if (len === 0) return;

      if (e.key === "j") {
        e.preventDefault();
        state.kbIndex = Math.min(len - 1, state.kbIndex + 1);
        highlightKeyboardRow();
      } else if (e.key === "k") {
        e.preventDefault();
        state.kbIndex = Math.max(0, state.kbIndex <= 0 ? 0 : state.kbIndex - 1);
        highlightKeyboardRow();
      } else if (e.key === "Enter" && state.kbIndex >= 0 && state.kbIndex < len) {
        e.preventDefault();
        const item = state.visibleItems[state.kbIndex];
        state.activeItemId = item.id;
        pushUrlState(false);
        renderAll();
        window.scrollTo({ top: 0, behavior: "smooth" });
      } else if (e.key === "o" && state.kbIndex >= 0 && state.kbIndex < len) {
        e.preventDefault();
        const item = state.visibleItems[state.kbIndex];
        window.open(item.url, "_blank", "noopener");
      } else if (e.key === "s" && state.kbIndex >= 0 && state.kbIndex < len) {
        e.preventDefault();
        const item = state.visibleItems[state.kbIndex];
        toggleSaveItem(item.id);
      }
    });
  }

  function highlightKeyboardRow() {
    const rows = document.querySelectorAll("#feed-list .news-row");
    rows.forEach((r, idx) => {
      const match = idx === state.kbIndex;
      r.classList.toggle("kb-focus", match);
      if (match) {
        r.scrollIntoView({ block: "nearest", behavior: "smooth" });
      }
    });
  }

  // Re-run KaTeX once deferred scripts finish loading
  window.addEventListener("load", () => {
    triggerMathRender(document.body);
  });

  // Boot
  initTheme();
  bindEvents();
  initData();
})();
