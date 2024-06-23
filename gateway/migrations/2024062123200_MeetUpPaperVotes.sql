CREATE TABLE IF NOT EXISTS meet_up_papers_votes (
    paper_id UUID NOT NULL,
    meet_up_id UUID NOT NULL,
    user_id UUID NOT NULL,
    vote INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (paper_id) REFERENCES papers(id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    PRIMARY KEY (paper_id, meet_up_id, user_id)
);
