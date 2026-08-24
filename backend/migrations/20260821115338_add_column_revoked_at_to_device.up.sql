-- Add up migration script here
ALTER TABLE devices
    ADD COLUMN revoked_at TIMESTAMPTZ;