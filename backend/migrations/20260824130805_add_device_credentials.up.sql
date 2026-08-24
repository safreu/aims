-- Add up migration script here
CREATE TABLE device_credentials (
    id UUID PRIMARY KEY,
    device_id UUID NOT NULL,
    token_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,

    CONSTRAINT device_credentials_device_fk
        FOREIGN KEY (device_id)
        REFERENCES devices (id)
        ON DELETE CASCADE
);

CREATE UNIQUE INDEX device_credentials_active_device_unique_idx
    ON device_credentials (device_id)
    WHERE revoked_at IS NULL;

CREATE UNIQUE INDEX device_credentials_token_hash_unique_idx
    ON device_credentials (token_hash);

CREATE INDEX device_credentials_device_id_idx
    ON device_credentials (device_id);