-- Add down migration script here
ALTER TABLE devices
    DROP COLUMN revoked_at;