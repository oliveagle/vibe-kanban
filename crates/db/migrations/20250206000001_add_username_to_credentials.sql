-- Add username column to user_credentials table for local auth support
-- This allows credentials to be stored by username instead of UUID

ALTER TABLE user_credentials ADD COLUMN IF NOT EXISTS username TEXT;

-- Create index for username lookups
CREATE INDEX IF NOT EXISTS idx_user_credentials_username
ON user_credentials (username);

-- Add unique constraint on username (for local auth users)
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_credentials_username_unique
ON user_credentials (username) WHERE username IS NOT NULL;
