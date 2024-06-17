CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY NOT NULL,
    nickname TEXT NOT NULL,
    email TEXT NOT NULL,
    login_method INT NOT NULL,
    access_token VARCHAR(32) NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX IF NOT EXISTS users_email_unique_index ON users (email);
CREATE INDEX IF NOT EXISTS users_access_token_index ON users (access_token);
