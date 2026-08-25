-- Add up migration script here
CREATE TABLE qr_actions (
    id UUID PRIMARY KEY,

    household_id UUID NOT NULL,
    item_id UUID NOT NULL,

    kind TEXT NOT NULL,
    amount BIGINT NOT NULL,

    revoked_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,

    CONSTRAINT qr_actions_household_fk
        FOREIGN KEY (household_id)
        REFERENCES households (id)
        ON DELETE CASCADE,

    CONSTRAINT qr_actions_item_fk
        FOREIGN KEY (item_id)
        REFERENCES inventory_items (id)
        ON DELETE RESTRICT,

    CONSTRAINT qr_actions_kind_valid
        CHECK (
            kind IN (
                'increase',
                'decrease'
            )
        ),

    CONSTRAINT qr_actions_amount_u32
        CHECK (
            amount > 0
            AND amount <= 4294967295
        )
);

CREATE INDEX qr_actions_household_id_idx
    ON qr_actions (household_id);

CREATE INDEX qr_actions_item_id_idx
    ON qr_actions (item_id);