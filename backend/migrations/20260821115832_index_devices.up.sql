-- Add up migration script here
CREATE INDEX devices_active_household_idx
    ON devices (household_id)
    WHERE revoked_at IS NULL;