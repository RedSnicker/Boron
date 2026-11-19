-- Preserve access to pre-existing channels while adding explicit private membership.
CREATE TABLE channel_members (
    channel_id TEXT NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY (channel_id, user_id)
);
INSERT OR IGNORE INTO channel_members (channel_id, user_id)
SELECT channels.id, team_members.user_id FROM channels
JOIN team_members ON team_members.team_id = channels.team_id;
CREATE TABLE channel_pages (
    channel_id TEXT PRIMARY KEY REFERENCES channels(id) ON DELETE CASCADE,
    content TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE issue_attachments (
    id TEXT PRIMARY KEY,
    issue_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    filename TEXT NOT NULL,
    data_url TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
-- Older code persisted Rust enum variant names instead of the API's snake_case.
UPDATE users SET role = lower(role);
UPDATE channels SET type = lower(type), required_role = lower(required_role);
UPDATE issues SET status = CASE status WHEN 'InProgress' THEN 'in_progress' ELSE lower(status) END, priority = lower(priority);
UPDATE messages SET message_type = CASE message_type WHEN 'GithubCommit' THEN 'github_commit' WHEN 'GithubPr' THEN 'github_pr' ELSE lower(message_type) END;
