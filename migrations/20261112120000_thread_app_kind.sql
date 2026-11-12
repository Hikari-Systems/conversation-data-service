-- Thread tags: `app` (the application that wrote the thread) and `kind` (the sort
-- of conversation it is within that application).
--
-- Both are nullable free text: this service does not know or validate the values,
-- which belong to its callers. A thread created without them stores NULL, and
-- `GET /api/thread/byUserId/{userId}?app=&kind=` filters never match NULL.

ALTER TABLE thread ADD COLUMN IF NOT EXISTS app VARCHAR(50);
ALTER TABLE thread ADD COLUMN IF NOT EXISTS kind VARCHAR(50);

CREATE INDEX IF NOT EXISTS thread_app_kind_idx ON thread (app, kind);
