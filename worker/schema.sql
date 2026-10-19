-- Ahbar — Cloudflare D1 / SQLite Schema for Passwordless Subscriptions & Signal/Noise Votes

CREATE TABLE IF NOT EXISTS subscribers (
  email TEXT PRIMARY KEY,
  mode TEXT NOT NULL DEFAULT 'hybrid',
  density TEXT NOT NULL DEFAULT 'compact',
  frequency TEXT NOT NULL DEFAULT 'daily',
  content_filter TEXT NOT NULL DEFAULT 'all',
  areas_json TEXT NOT NULL DEFAULT '["cs","ai","math","physics","systems"]',
  tags_json TEXT NOT NULL DEFAULT '[]',
  verified INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS item_votes (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  item_id TEXT NOT NULL,
  vote TEXT NOT NULL CHECK (vote IN ('signal', 'noise')),
  voter_hash TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE(item_id, voter_hash)
);

CREATE INDEX IF NOT EXISTS idx_item_votes_item_id ON item_votes(item_id);
