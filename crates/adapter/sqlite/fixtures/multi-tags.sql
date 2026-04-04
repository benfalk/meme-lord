-- Add multiple tags to the database for testing purposes.

INSERT INTO user_tags (id, owner_id, name)
VALUES (
  X'01890f4e5b7a7cc2bb7b8e8b8e7e9c3e',
  X'01890f4d2e3b7e3a8c1d9b2a4f6e1d2c',
  "nsfw"
);

INSERT INTO user_tags (id, owner_id, name)
VALUES (
  X'01890f4e5b7a7cc2bb7b8e8b8e7e9c4a',
  X'01890f4d2e3b7e3a8c1d9b2a4f6e1d2c',
  "programming"
);
