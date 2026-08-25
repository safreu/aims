-- Add up migration script here

ALTER TABLE inventory_items
ADD CONSTRAINT inventory_items_id_household_unique
    UNIQUE (id, household_id);
    
CREATE TABLE inventory_shopping_states (
    household_id UUID NOT NULL,
    item_id UUID NOT NULL,

    quantity_override BIGINT,
    note VARCHAR(50),
    checked BOOLEAN NOT NULL DEFAULT FALSE,
    dismissed BOOLEAN NOT NULL DEFAULT FALSE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (household_id, item_id),

    CONSTRAINT inventory_shopping_states_item_fk
        FOREIGN KEY (item_id, household_id)
        REFERENCES inventory_items (id, household_id)
        ON DELETE CASCADE,

    CONSTRAINT inventory_shopping_states_quantity_override_positive
        CHECK (
            quantity_override IS NULL
            OR quantity_override > 0
        ),

    CONSTRAINT inventory_shopping_states_quantity_override_u32
        CHECK (
            quantity_override IS NULL
            OR quantity_override <= 4294967295
        )
);