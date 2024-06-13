CREATE TABLE IF NOT EXISTS future_meet_ups (
    id UUID PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    state INT NOT NULL,
    description TEXT NULL,
    speaker TEXT NULL,
    date DATE NOT NULL
);
