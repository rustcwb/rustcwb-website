CREATE TABLE IF NOT EXISTS github_logins (
    id UUID PRIMARY KEY NOT NULL,
    user_id UUID NOT NULL,
    access_token TEXT NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    refresh_token TEXT NOT NULL,
    refresh_token_expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
CREATE UNIQUE INDEX IF NOT EXISTS github_logins_user_id_index ON github_logins (user_id);
