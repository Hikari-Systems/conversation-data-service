ALTER TABLE message ADD COLUMN role VARCHAR(20);
ALTER TABLE message ADD COLUMN tool_call_json JSONB;

UPDATE message SET role = 'user' WHERE length(sender_id) = 36;
UPDATE message SET role = 'assistant' WHERE length(sender_id) <> 36;
