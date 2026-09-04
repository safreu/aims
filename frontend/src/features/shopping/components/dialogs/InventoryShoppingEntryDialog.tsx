import { useEffect, useRef, useState, type SubmitEvent } from "react";
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
  const dialogRef = useRef<HTMLDialogElement>(null);

  const [quantity, setQuantity] = useState(entry.quantity);
  const [note, setNote] = useState(entry.note ?? "");

  const [isMutating, setIsMutating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    dialogRef.current?.showModal();
  }, []);

  async function handleSave(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    setIsMutating(true);
    setError(null);

    await Promise.all([
      setShoppingQuantity(householdId, entry.item_id, { quantity }),
      setShoppingNote(householdId, entry.item_id, {
        note: note.trim() === "" ? null : note.trim(),
      })
        .then(async () => {
          await onChanged();
          dialogRef.current?.close();
        })
        .catch(() => setError("Failed to update shopping item"))
        .finally(() => setIsMutating(false)),
    ]);
  }

  async function handleDismiss() {
    setIsMutating(true);
    setError(null);

    await dismissShoppingItem(householdId, entry.item_id)
      .catch(() => setError("Failed to dismiss shopping item"))
      .then(async () => {
        await onChanged();
        dialogRef.current?.close();
      })
      .finally(() => setIsMutating(false));
  }

  return (
    <dialog
      ref={dialogRef}
      className="inventory-item-dialog"
      onClose={onClose}
      onClick={(event) => {
        if (event.target === dialogRef.current) {
          dialogRef.current?.close();
        }
      }}
    >
      <div className="inventory-item-dialog__content">
        <header className="inventory-item-dialog__header">
          <div>
            <h2>{entry.name}</h2>
            <p>Edit this shopping item</p>
          </div>

          <button
            type="button"
            className="button button--ghost inventory-item-dialog__close"
            onClick={() => dialogRef.current?.close()}
          >
            Close
          </button>
        </header>

        <form className="inventory-item-dialog__section" onSubmit={handleSave}>
          <div className="inventory-item-dialog__fields">
            <label className="inventory-item-dialog__field">
              <span>Quantity</span>

              <input
                type="number"
                min="1"
                value={quantity}
                onChange={(event) => setQuantity(Number(event.target.value))}
                disabled={isMutating}
                required
              />
            </label>

            <label className="inventory-item-dialog__field">
              <span>Note</span>

              <input
                type="text"
                value={note}
                onChange={(event) => setNote(event.target.value)}
                disabled={isMutating}
              />
            </label>
          </div>

          <button
            type="submit"
            className="button button--primary"
            disabled={isMutating}
          >
            {isMutating ? "Saving..." : "Save"}
          </button>
        </form>

        <section className="inventory-item-dialog__section inventory-item-dialog__danger">
          <h3>Dismiss item</h3>

          <p>Remove this inventory item from the shopping list</p>

          <button
            type="button"
            className="button button--danger"
            disabled={isMutating}
            onClick={() => void handleDismiss()}
          >
            Dismiss
          </button>
        </section>

        {error !== null && (
          <p className="inventory-item-dialog__error">{error}</p>
        )}
      </div>
    </dialog>
  );
}
