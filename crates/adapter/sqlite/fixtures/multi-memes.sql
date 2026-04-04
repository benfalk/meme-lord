-- contains the single meme along with another both
-- by the same owner_id

INSERT INTO memes (id, owner_id, meme_path, caption, file_size)
VALUES (
  X'01890f4c5b7a7cc2bb7b8e8b8e7e9c3a',
  X'01890f4d2e3b7e3a8c1d9b2a4f6e1d2c',
  "/something-of-a-meme-lord-myself.jpg",
  "I'm something of a meme lord myself",
  127000
);

INSERT INTO memes (id, owner_id, meme_path, caption, file_size)
VALUES (
  X'01890f4c5b7a7cc2bb7b8e8b8e7e9c3b',
  X'01890f4d2e3b7e3a8c1d9b2a4f6e1d2c',
  "/another-swing.jpg",
  null,
  588
);
