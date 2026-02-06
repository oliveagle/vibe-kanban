-- Add user_credentials table for storing OAuth tokens in database
-- This allows credentials to be synchronized across multiple devices

CREATE TABLE IF NOT EXISTS user_credentials (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL,
    access_token    TEXT,
    refresh_token   TEXT NOT NULL,
    expires_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index for faster lookups by user_id
CREATE INDEX IF NOT EXISTS idx_user_credentials_user_id
ON user_credentials (user_id);

-- Add unique constraint to ensure one credential record per user
-- (Can be relaxed later if we want to support multiple OAuth providers per user)
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_credentials_user_id_unique
ON user_credentials (user_id);

-- Create trigger to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION update_user_credentials_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_user_credentials_updated_at ON user_credentials;
CREATE TRIGGER trg_user_credentials_updated_at
    BEFORE UPDATE ON user_credentials
    FOR EACH ROW
    EXECUTE FUNCTION update_user_credentials_updated_at();
