CREATE TABLE stored_files (
    id TEXT PRIMARY KEY,
    scope TEXT NOT NULL,
    parent_id TEXT NOT NULL,
    uploader_id TEXT NOT NULL REFERENCES users(id),
    filename TEXT NOT NULL,
    mime TEXT NOT NULL,
    size INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_stored_files_parent ON stored_files(scope, parent_id);
CREATE TABLE message_reactions (
    message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    emoji TEXT NOT NULL,
    PRIMARY KEY (message_id, user_id, emoji)
);
