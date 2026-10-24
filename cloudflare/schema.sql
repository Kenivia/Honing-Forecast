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
