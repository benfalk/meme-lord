CREATE TABLE memes (
    id BLOB PRIMARY KEY,
    owner_id BLOB NOT NULL,
    meme_path TEXT NOT NULL,
    caption TEXT,
    file_size INTEGER NOT NULL
);

CREATE INDEX idx_memes_owner_id ON memes (owner_id);
CREATE UNIQUE INDEX idx_memes_meme_path ON memes (meme_path);
