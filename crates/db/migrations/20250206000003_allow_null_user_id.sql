-- Allow NULL user_id for local auth credentials
ALTER TABLE user_credentials ALTER COLUMN user_id DROP NOT NULL;
