ALTER TABLE message RENAME COLUMN tool_call_json TO tool_calls;
ALTER TABLE message ADD COLUMN tool_result_call_id VARCHAR(36);
ALTER TABLE message DROP COLUMN role;
