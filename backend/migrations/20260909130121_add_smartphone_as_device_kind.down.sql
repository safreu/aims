-- Add down migration script here
ALTER TABLE devices
DROP CONSTRAINT devices_kind_valid;

ALTER TABLE devices
ADD CONSTRAINT devices_kind_valid
CHECK (kind IN ('scanner', 'display', 'other'));