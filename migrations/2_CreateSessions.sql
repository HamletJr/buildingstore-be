CREATE TABLE IF NOT EXISTS sessions (
    session_key UUID PRIMARY KEY,
    user_id INTEGER NOT NULL,
    expires_at TIMESTAMP NOT NULL
);