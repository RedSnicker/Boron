-- Preserve existing single-person assignments while supporting many assignees.
CREATE TABLE issue_assignees (
    issue_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    position INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (issue_id, user_id)
);
INSERT OR IGNORE INTO issue_assignees (issue_id, user_id, position)
SELECT id, assignee_id, 0 FROM issues WHERE assignee_id IS NOT NULL;
CREATE INDEX idx_issue_assignees_user ON issue_assignees(user_id);
