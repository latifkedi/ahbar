/* Ahbar Service Worker — Offline Reading & Instant Repeat Visits */
const CACHE_NAME = "ahbar-cache-v1";
const STATIC_ASSETS = [
  "./",
  "index.html",
  "style.css",
  "boot.js",
  "app.js",
  "manifest.json",
  "fonts/serif-400.woff2",
  "fonts/serif-600.woff2",
  "data/tags.json",
  "data/archive.json",
  "data/latest.tr.json",
  "data/latest.en.json"
];

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => {
      return cache.addAll(STATIC_ASSETS).catch((err) => {
        console.warn("[SW] Cache preload warning:", err);
      });
    }).then(() => self.skipWaiting())
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches.keys().then((keys) => {
      return Promise.all(
        keys.filter((key) => key !== CACHE_NAME).map((key) => caches.delete(key))
      );
    }).then(() => self.clients.claim())
  );
});

self.addEventListener("fetch", (event) => {
  const req = event.request;
  if (req.method !== "GET") return;

  const url = new URL(req.url);

  // For data JSON and HTML: Network first, fall back to cache when offline
  if (url.pathname.endsWith(".json") || url.pathname.endsWith(".html") || url.pathname === "/" || url.pathname.endsWith("/ahbar/")) {
    event.respondWith(
      fetch(req)
        .then((res) => {
          if (res.ok) {
            const clone = res.clone();
            caches.open(CACHE_NAME).then((cache) => cache.put(req, clone));
          }
          return res;
        })
        .catch(() => caches.match(req))
    );
    return;
  }

  // For static fonts, CSS, scripts: Cache first, fall back to network
  event.respondWith(
    caches.match(req).then((cached) => {
      if (cached) return cached;
      return fetch(req).then((res) => {
        if (res.ok) {
          const clone = res.clone();
          caches.open(CACHE_NAME).then((cache) => cache.put(req, clone));
        }
        return res;
      });
    })
  );
});
