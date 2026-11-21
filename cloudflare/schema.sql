-- one row per upload: a roster's snapshots not sent before, and its characters' recent raids
CREATE TABLE IF NOT EXISTS uploads (
  -- never reused, even after the pull empties the table
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  received INTEGER,
  roster TEXT,
  region TEXT,
  version TEXT,
  snapshots TEXT,
  raids TEXT,
  -- gear levels per character from lostark.bible (absent before 2026-10-24: ALTER TABLE uploads ADD COLUMN gear TEXT)
  gear TEXT
);

-- per exported character, when it is looked up again; names are kept only here, until then
CREATE TABLE IF NOT EXISTS pending (
  char TEXT PRIMARY KEY,
  roster TEXT,
  region TEXT,
  version TEXT,
  name TEXT,
  due INTEGER
);

-- upstream fetches in the current minute, per source
CREATE TABLE IF NOT EXISTS limits (
  source TEXT PRIMARY KEY,
  minute INTEGER,
  n INTEGER
);

-- how each browser's domain move went (frontend/Utils/MoveReport.ts); no names, no save contents
CREATE TABLE IF NOT EXISTS moves (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  received INTEGER,
  site TEXT,
  version TEXT,
  outcome TEXT,
  error TEXT,
  chunks INTEGER,
  bytes INTEGER
);

-- market scanner passes (crates/market), each as uploaded: {version, region, started, finished, sections, points}
CREATE TABLE IF NOT EXISTS market_scans (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  region TEXT,
  started TEXT,
  finished TEXT,
  received INTEGER,
  body TEXT,
  -- a pass is stored once; also serves the latest pass per region
  UNIQUE (region, started)
);

-- the newest lowest listing of each material per region, over every pass: {slug: [price, at]}
CREATE TABLE IF NOT EXISTS market_latest (
  region TEXT PRIMARY KEY,
  body TEXT
);

-- the passes' points per region and month: {key: {day: value}}
CREATE TABLE IF NOT EXISTS market_history (
  region TEXT,
  month TEXT,
  body TEXT,
  PRIMARY KEY (region, month)
);
