import { useEffect, useRef, useState, type SubmitEvent } from "react";
import type { CustomShoppingEntry, ShoppingPriority } from "../../types";
import {
  deleteCustomShoppingEntry,
  updateCustomShoppingEntry,
} from "../../api";
import {
  Select,
  type SelectOption,
} from "../../../../components/select/Select";

type CustomShoppingEntryDialogProps = {
  householdId: string;
  entry: CustomShoppingEntry;
  onChanged: () => Promise<void>;
  onClose: () => void;
};

const PriorityOptions: SelectOption<ShoppingPriority>[] = [
  { value: "default", label: "Default" },
  { value: "low", label: "Low" },
  { value: "medium", label: "Medium" },
  { value: "high", label: "High" },
];

export function CustomShoppingEntryDialog({
  householdId,
  entry,
  onChanged,
  onClose,
}: CustomShoppingEntryDialogProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);

  const [title, setTitle] = useState(entry.title);
  const [quantity, setQuantity] = useState(entry.quantity);
  const [priority, setPriority] = useState(entry.priority);
  const [note, setNote] = useState(entry.note ?? "");

  const [isMutating, setIsMutating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    dialogRef.current?.showModal();
  }, []);

  async function handleSave(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    const trimmedTitle = title.trim();

    if (trimmedTitle === "") {
      setError("Title is required");
      return;
    }

    setIsMutating(true);
    setError(null);

    await updateCustomShoppingEntry(householdId, entry.id, {
      title: trimmedTitle,
      quantity,
      priority,
      note: note.trim() === "" ? null : note.trim(),
    })
      .catch(() => setError("Failed to update shopping item"))
      .then(async () => {
        await onChanged();
        dialogRef.current?.close();
      })
      .finally(() => setIsMutating(false));
  }

  async function handleDelete() {
    setIsMutating(true);
    setError(null);

    await deleteCustomShoppingEntry(householdId, entry.id)
      .then(async () => {
        await onChanged();
        dialogRef.current?.close();
      })
      .catch(() => setError("Failed to dismiss shopping item"))
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
            <h2>{entry.title}</h2>
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

        <form onSubmit={handleSave}>
          <section className="inventory-item-dialog__section">
            <div className="inventory-item-dialog__fields">
              <label className="inventory-item-dialog__field">
                <span>Name</span>

                <input
                  type="text"
                  value={title}
                  disabled={isMutating}
                  onChange={(event) => setTitle(event.target.value)}
                />
              </label>

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
                <span>Priority</span>

                <Select
                  value={priority}
                  options={PriorityOptions}
                  onValueChange={(value) => setPriority(value)}
                  portal={false}
                  disabled={isMutating}
                  ariaLabel="Priority"
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

            {error !== null && (
              <p className="inventory-item-dialog__error">{error}</p>
            )}

            <button
              type="submit"
              className="button button--primary"
              disabled={isMutating}
            >
              {isMutating ? "Saving..." : "Save"}
            </button>
          </section>
        </form>

        <section className="inventory-item-dialog__section inventory-item-dialog__danger">
          <h3>Delete item</h3>

          <p>Permanently remove this custom item from the shopping list</p>

          <button
            type="button"
            className="button button--danger"
            disabled={isMutating}
            onClick={() => void handleDelete()}
          >
            Delete
          </button>
        </section>
      </div>
    </dialog>
  );
}
