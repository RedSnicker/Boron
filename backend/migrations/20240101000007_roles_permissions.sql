-- Roles table for Discord-like role system
CREATE TABLE IF NOT EXISTS roles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    team_id TEXT NOT NULL,
    name TEXT NOT NULL,
    color TEXT DEFAULT '#7289da',
    permissions TEXT DEFAULT '[]',
    created_at TEXT NOT NULL
);

-- User-role assignments
CREATE TABLE IF NOT EXISTS user_roles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    role_id INTEGER NOT NULL,
    assigned_at TEXT NOT NULL,
    FOREIGN KEY (role_id) REFERENCES roles(id)
);

-- Channel-specific permissions (override role permissions)
CREATE TABLE IF NOT EXISTS channel_permissions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    channel_id TEXT NOT NULL,
    role_id INTEGER,
    user_id TEXT,
    permissions TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    FOREIGN KEY (role_id) REFERENCES roles(id)
);

-- Categories for organizing channels
CREATE TABLE IF NOT EXISTS categories (
    id TEXT PRIMARY KEY,
    team_id TEXT NOT NULL,
    name TEXT NOT NULL,
    position INTEGER DEFAULT 0,
    created_at TEXT NOT NULL
);

-- Add category_id and channel_type_v2 to channels
ALTER TABLE channels ADD COLUMN category_id TEXT;
ALTER TABLE channels ADD COLUMN channel_type_v2 TEXT DEFAULT 'text';

-- Add position and icon to channels (if not already present)
ALTER TABLE channels ADD COLUMN position INTEGER DEFAULT 0;
ALTER TABLE channels ADD COLUMN icon TEXT;

-- Add user status and custom status
ALTER TABLE users ADD COLUMN status TEXT DEFAULT 'online';
ALTER TABLE users ADD COLUMN custom_status TEXT;

-- Add is_private to issues (if not already present)
ALTER TABLE issues ADD COLUMN is_private INTEGER DEFAULT 0;