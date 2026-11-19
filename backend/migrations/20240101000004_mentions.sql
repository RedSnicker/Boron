CREATE TABLE message_mentions (
    message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    read_at TEXT,
    PRIMARY KEY (message_id, user_id)
);
CREATE INDEX idx_mentions_unread ON message_mentions(user_id, read_at);
