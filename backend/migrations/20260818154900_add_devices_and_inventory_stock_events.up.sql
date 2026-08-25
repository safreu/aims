-- Add up migration script here
CREATE TABLE devices (
    id UUID PRIMARY KEY,
    household_id UUID NOT NULL,
    name TEXT NOT NULL,
    kind TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,

    CONSTRAINT devices_household_fk
        FOREIGN KEY (household_id)
        REFERENCES households (id)
        ON DELETE CASCADE,

    CONSTRAINT devices_kind_valid
        CHECK (
            kind IN (
                'scanner',
                'display',
                'other'
            )
        )
);

CREATE INDEX devices_household_id_idx
    ON devices (household_id);


CREATE TABLE inventory_stock_events (
    id UUID PRIMARY KEY,

    sequence_number BIGINT GENERATED ALWAYS AS IDENTITY,

    household_id UUID NOT NULL,
    item_id UUID NOT NULL,

    actor_user_id UUID,
    actor_device_id UUID,

    kind TEXT NOT NULL,
    source TEXT NOT NULL,

    amount BIGINT,

    stock_before BIGINT NOT NULL,
    stock_after BIGINT NOT NULL,

    created_at TIMESTAMPTZ NOT NULL,

    CONSTRAINT inventory_stock_events_household_fk
        FOREIGN KEY (household_id)
        REFERENCES households (id)
        ON DELETE CASCADE,

    CONSTRAINT inventory_stock_events_item_fk
        FOREIGN KEY (item_id)
        REFERENCES inventory_items (id)
        ON DELETE RESTRICT,

    CONSTRAINT inventory_stock_events_actor_user_fk
        FOREIGN KEY (actor_user_id)
        REFERENCES users (id)
        ON DELETE SET NULL,

    CONSTRAINT inventory_stock_events_actor_device_fk
        FOREIGN KEY (actor_device_id)
        REFERENCES devices (id)
        ON DELETE SET NULL,

    CONSTRAINT inventory_stock_events_single_actor
        CHECK (
            NOT (
                actor_user_id IS NOT NULL
                AND actor_device_id IS NOT NULL
            )
        ),

    CONSTRAINT inventory_stock_events_kind_valid
        CHECK (
            kind IN (
                'increase',
                'decrease',
                'set'
            )
        ),

    CONSTRAINT inventory_stock_events_source_valid
        CHECK (
            source IN (
                'manual',
                'qr',
                'barcode',
                'system'
            )
        ),

    CONSTRAINT inventory_stock_events_amount_valid
        CHECK (
            (
                kind IN ('increase', 'decrease')
                AND amount IS NOT NULL
                AND amount > 0
                AND amount <= 4294967295
            )
            OR
            (
                kind = 'set'
                AND amount IS NULL
            )
        ),

    CONSTRAINT inventory_stock_events_stock_before_u32
        CHECK (
            stock_before >= 0
            AND stock_before <= 4294967295
        ),

    CONSTRAINT inventory_stock_events_stock_after_u32
        CHECK (
            stock_after >= 0
            AND stock_after <= 4294967295
        )
);

CREATE UNIQUE INDEX inventory_stock_events_sequence_number_unique_idx
    ON inventory_stock_events (sequence_number);

CREATE INDEX inventory_stock_events_household_id_idx
    ON inventory_stock_events (household_id);

CREATE INDEX inventory_stock_events_item_id_idx
    ON inventory_stock_events (item_id);

CREATE INDEX inventory_stock_events_created_at_idx
    ON inventory_stock_events (created_at);

CREATE INDEX inventory_stock_events_item_sequence_idx
    ON inventory_stock_events (
        item_id,
        sequence_number DESC
    );