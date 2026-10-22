-- one row per upload: a roster's snapshots not sent before, and its characters' recent raids
CREATE TABLE IF NOT EXISTS uploads (
  -- never reused, even after the pull empties the table
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  received INTEGER,
  roster TEXT,
  region TEXT,
  version TEXT,
  snapshots TEXT,
  raids TEXT
);
