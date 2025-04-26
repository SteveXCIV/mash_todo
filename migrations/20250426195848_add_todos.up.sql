CREATE TABLE IF NOT EXISTS todos (
  -- this is an alias for the row's unique ID (see: https://www.sqlite.org/autoinc.html)
  id INTEGER PRIMARY KEY NOT NULL,
  description TEXT NOT NULL,
  completed_at BIGINT
);