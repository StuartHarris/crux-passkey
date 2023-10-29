--
-- user name to user id
CREATE TABLE IF NOT EXISTS user (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_name TEXT NOT NULL,
  user_id BLOB NOT NULL
);
CREATE INDEX IF NOT EXISTS user_user_name ON user (user_name);
--
-- user id to credentials
CREATE TABLE IF NOT EXISTS credentials (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id BLOB NOT NULL,
  credentials BLOB NOT NULL
);
CREATE INDEX IF NOT EXISTS credentials_user_id ON credentials (user_id);
--
-- session id to session
CREATE TABLE IF NOT EXISTS user_session (
  -- Uuid
  id BLOB PRIMARY KEY,
  -- serialized Session
  data BLOB NOT NULL
);
