CREATE TABLE thread (
    id UUID PRIMARY KEY NOT NULL,
    title VARCHAR(400) NOT NULL,
    visible_to_user_ids VARCHAR(100)[],
    bot_id VARCHAR(100),
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
);

CREATE INDEX thread_visibletouserids_idx ON thread USING GIN (visible_to_user_ids);

CREATE TABLE message (
    id UUID PRIMARY KEY NOT NULL,
    thread_id UUID NOT NULL REFERENCES thread(id),
    sender_id VARCHAR NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
);

CREATE INDEX message_thread_id_idx ON message (thread_id);
CREATE INDEX message_sender_id_idx ON message (sender_id);
