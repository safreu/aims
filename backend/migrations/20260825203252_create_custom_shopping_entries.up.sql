-- Add up migration script here
CREATE TABLE custom_shopping_entries (
    id UUID PRIMARY KEY,
    household_id UUID NOT NULL,
    title TEXT NOT NULL,
    quantity BIGINT NOT NULL,
    priority TEXT NOT NULL,
    note TEXT,
    checked BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,

    CONSTRAINT custom_shopping_entries_household_fk
        FOREIGN KEY (household_id)
        REFERENCES households (id)
        ON DELETE CASCADE,

    CONSTRAINT custom_shopping_entries_quantity_positive
        CHECK (quantity > 0),

    CONSTRAINT custom_shopping_entries_note_length
        CHECK (note IS NULL OR char_length(note) <= 50),

    CONSTRAINT custom_shopping_entries_timestamps_valid
        CHECK (updated_at >= created_at)
);

CREATE INDEX custom_shopping_entries_household_id_idx
    ON custom_shopping_entries (household_id);