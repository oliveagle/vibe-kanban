-- Fix unique constraints on user_credentials table
-- Add proper unique constraints for ON CONFLICT support

-- Drop old partial unique indexes
DROP INDEX IF EXISTS idx_user_credentials_user_id_unique;
DROP INDEX IF EXISTS idx_user_credentials_username_unique;

-- Add proper unique constraints
ALTER TABLE user_credentials ADD CONSTRAINT uq_user_credentials_user_id UNIQUE (user_id);
ALTER TABLE user_credentials ADD CONSTRAINT uq_user_credentials_username UNIQUE (username);
