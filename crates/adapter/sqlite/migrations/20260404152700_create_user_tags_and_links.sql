CREATE TABLE user_tags (
  id BLOB PRIMARY KEY,
  owner_id BLOB NOT NULL,
  name TEXT NOT NULL
);

CREATE UNIQUE INDEX idx_user_tags_owner_id_name ON user_tags (owner_id, name);

CREATE TABLE user_tag_links (
  tag_id BLOB NOT NULL,
  meme_id BLOB NOT NULL,
  FOREIGN KEY (tag_id) REFERENCES user_tags (id) ON DELETE CASCADE,
  FOREIGN KEY (meme_id) REFERENCES memes (id) ON DELETE CASCADE
  PRIMARY KEY (tag_id, meme_id)
);
