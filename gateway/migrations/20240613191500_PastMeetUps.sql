CREATE TABLE IF NOT EXISTS past_meet_ups (
    id UUID PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    speaker TEXT NOT NULL,
    date DATE NOT NULL,
    link TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS past_meet_ups_date_index ON past_meet_ups (date);