CREATE TABLE users (
  id BLOB PRIMARY KEY,
  name TEXT NOT NULL
);

CREATE UNIQUE INDEX idx_users_name_nocase ON users(name COLLATE NOCASE);

CREATE TABLE user_password_hashes (
  user_id BLOB PRIMARY KEY,
  password_hash BLOB NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);
