import { useState, type SubmitEvent } from "react";

import { Dialog } from "../../../../components/dialog/Dialog";
import { Select } from "../../../../components/select/Select";
import { useToast } from "../../../../components/toast/ToastContext";
import { PRIORITIES, type Priority } from "../../../../domain/priority";
import { updateInventoryItem } from "../../../inventory/api";
import {
  dismissShoppingItem,
  setShoppingNote,
  setShoppingQuantity,
} from "../../api";
import type { InventoryShoppingEntry } from "../../types";

type InventoryShoppingEntryDialogProps = {
  householdId: string;
  entry: InventoryShoppingEntry;
  onChanged: () => Promise<void>;
  onClose: () => void;
};

export function InventoryShoppingEntryDialog({
  householdId,
  entry,
  onChanged,
  onClose,
}: InventoryShoppingEntryDialogProps) {
  const { showToast } = useToast();

  const [quantity, setQuantity] = useState<number | "">(entry.quantity);
  const [priority, setPriority] = useState<Priority>(entry.priority);
  const [note, setNote] = useState(entry.note ?? "");

  const [isMutating, setIsMutating] = useState(false);
  const [quantityError, setQuantityError] = useState<string>();

  async function handleSave(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    if (quantity === "") {
      setQuantityError("Quantity is required");
      return;
    }

    if (!Number.isInteger(quantity) || quantity < 1) {
      setQuantityError("Quantity must be a positive whole number");
      return;
    }

    setQuantityError(undefined);
    setIsMutating(true);

    try {
      await Promise.all([
        setShoppingQuantity(householdId, entry.item_id, { quantity }),
        updateInventoryItem(householdId, entry.item_id, { priority }),
        setShoppingNote(householdId, entry.item_id, {
          note: note.trim() === "" ? null : note.trim(),
        }),
      ]);

      await onChanged();

      onClose();
      showToast("Shopping item updated", "success");
    } catch {
      showToast("Failed to update shopping item", "error");
    } finally {
      setIsMutating(false);
    }
  }

  async function handleDismiss() {
    setIsMutating(true);

    try {
      await dismissShoppingItem(householdId, entry.item_id);

      await onChanged();

      onClose();
      showToast("Shopping item dismissed", "success");
    } catch {
      showToast("Failed to dismiss shopping item", "error");
    } finally {
      setIsMutating(false);
    }
  }

  return (
    <Dialog
      title={entry.name}
      description="Edit this shopping item"
      onClose={onClose}
      closeDisabled={isMutating}
    >
      <form className="dialog__section" onSubmit={handleSave}>
        <div className="dialog__fields">
          <label
            className={`dialog__field ${
              quantityError ? "dialog__field--error" : ""
            }`}
          >
            <span>Quantity</span>

            <input
              type="number"
              min="1"
              step="1"
              value={quantity}
              onChange={(event) => {
                const value = event.target.value;

                setQuantity(value === "" ? "" : Number(value));

                if (quantityError !== undefined) {
                  setQuantityError(undefined);
                }
              }}
              disabled={isMutating}
            />

            {quantityError && (
              <span className="dialog__field-error" role="alert">
                {quantityError}
              </span>
            )}
          </label>

          <label className="dialog__field">
            <span>Priority</span>

            <Select
              value={priority}
              options={PRIORITIES}
              onValueChange={(value) => setPriority(value as Priority)}
              portal={false}
              disabled={isMutating}
              ariaLabel="Priority"
            />
          </label>

          <label className="dialog__field">
            <span>Note</span>

            <input
              type="text"
              value={note}
              onChange={(event) => setNote(event.target.value)}
              disabled={isMutating}
            />
          </label>
        </div>

        <div className="dialog__actions">
          <button
            type="submit"
            className="button button--primary"
            disabled={isMutating}
          >
            {isMutating ? "Saving..." : "Save"}
          </button>
        </div>
      </form>

      <section className="dialog__section dialog__danger">
        <h3>Dismiss item</h3>

        <p>Remove this inventory item from the shopping list</p>

        <div className="dialog__actions">
          <button
            type="button"
            className="button button--danger"
            disabled={isMutating}
            onClick={() => void handleDismiss()}
          >
            Dismiss
          </button>
        </div>
      </section>
    </Dialog>
  );
}
