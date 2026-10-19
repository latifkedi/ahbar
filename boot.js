/* Runs in <head> before first paint (modeled on useful-sites/boot.js):
   1. Applies saved dark theme immediately so dark-mode readers never see a white flash.
   2. Adds html.pending when opening a filtered URL (?area=, ?item=, ?q=, ?cmp=, ?s=)
      so the pre-rendered home block shows a clean skeleton until app.js renders.
   3. Detects the visitor's active language (TR or EN) from ?lang=, localStorage,
      or navigator.language, and pre-fetches only that language's JSON payload
      (data/latest.tr.json or data/latest.en.json) in parallel with page parsing. */
(function () {
  var d = document,
    s = d.currentScript,
    B = (window.__boot = { done: false, lang: "tr", issuePromise: null });

  function pref(k) {
    try {
      return localStorage.getItem(k);
    } catch (_) {
      return null;
    }
  }

  try {
    var th = pref("ahbar.theme.v2") || pref("theme");
    if (th === "dark" || th === "light") {
      d.documentElement.setAttribute("data-theme", th);
    }
  } catch (_) {}

  try {
    if (/[?&](?!lang=)[a-z]+=/.test(location.search)) {
      d.documentElement.classList.add("pending");
    }
  } catch (_) {}

  var lang = null;
  try {
    lang = new URLSearchParams(location.search).get("lang");
  } catch (_) {}
  if (lang !== "tr" && lang !== "en") {
    lang = pref("ahbar.lang.v2") || pref("lang");
  }
  if (lang !== "tr" && lang !== "en") {
    var navLang = (navigator.language || "").toLowerCase();
    lang = navLang.indexOf("tr") === 0 ? "tr" : "en";
  }
  B.lang = lang;
  d.documentElement.lang = lang;

  var src = s && s.getAttribute("data-" + lang);
  if (src && typeof window.fetch === "function") {
    B.issuePromise = fetch(src, { cache: "no-cache" })
      .then(function (r) {
        if (!r.ok) throw new Error("HTTP " + r.status);
        return r.json();
      })
      .catch(function () {
        return null;
      });
  }
})();
