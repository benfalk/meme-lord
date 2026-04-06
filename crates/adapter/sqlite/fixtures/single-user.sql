INSERT INTO users (id, name)
VALUES(
  X'01890f4d2e3b7e3a8c1d9b2a4f6e1d2c',
  "bman"
);

-- password hash is the same as the user id for simplicity, but in a
-- real application, it should be a securely hashed password.
INSERT INTO user_password_hashes (user_id, password_hash)
VALUES(
  X'01890f4d2e3b7e3a8c1d9b2a4f6e1d2c',
  X'01890f4d2e3b7e3a8c1d9b2a4f6e1d2c'
);
