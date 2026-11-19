ALTER TABLE users ADD COLUMN bio TEXT NOT NULL DEFAULT '';
ALTER TABLE messages ADD COLUMN edited_at TEXT;
ALTER TABLE messages ADD COLUMN is_pinned INTEGER NOT NULL DEFAULT 0;
CREATE TABLE message_reminders (
 id TEXT PRIMARY KEY,
 user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
 message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
 remind_at TEXT NOT NULL,
 delivered_at TEXT,
 created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_reminders_due ON message_reminders(user_id, delivered_at, remind_at);
