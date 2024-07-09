CREATE TABLE IF NOT EXISTS meet_up_goers (
    user_id UUID NOT NULL,
    meet_up_id UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, meet_up_id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (meet_up_id) REFERENCES meet_ups(id)
);