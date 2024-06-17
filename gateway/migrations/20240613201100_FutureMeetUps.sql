CREATE TABLE IF NOT EXISTS future_meet_ups (
    id UUID PRIMARY KEY NOT NULL,
    title TEXT NULL,
    state INT NOT NULL,
    description TEXT NULL,
    speaker TEXT NULL,
    date DATE NOT NULL,
    location TEXT NOT NULL
);
