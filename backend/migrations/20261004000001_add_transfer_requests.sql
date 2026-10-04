ALTER TABLE users ADD COLUMN reset_token TEXT;

CREATE TABLE transfer_requests (
    id TEXT PRIMARY KEY,
    user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
    secret_code TEXT NOT NULL UNIQUE,
    name TEXT,
    description TEXT,
    active BOOLEAN DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
