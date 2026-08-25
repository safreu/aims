-- Add down migration script here
DROP TABLE inventory_shopping_states;

ALTER TABLE inventory_items
DROP CONSTRAINT inventory_items_id_household_unique;