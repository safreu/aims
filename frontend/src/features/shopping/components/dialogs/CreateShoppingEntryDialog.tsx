import { useEffect, useRef, useState, type SubmitEvent } from "react";
import type { InventoryItemPriority } from "../../../inventory/types";
import { createCustomShoppingEntry } from "../../api";
import { ShoppingEntryFields } from "../fields/ShoppingEntryFields";
import { useToast } from "../../../../components/toast/ToastContext";

type CreateShoppingEntryDialogProps = {
  householdId: string;
  onCreated: () => Promise<void>;
  onClose: () => void;
};

export function CreateShoppingEntryDialog({
  householdId,
  onCreated,
  onClose,
}: CreateShoppingEntryDialogProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);

  const { showToast } = useToast();

  const [title, setTitle] = useState("");
  const [quantity, setQuantity] = useState(1);
  const [priority, setPriority] = useState<InventoryItemPriority>("default");
  const [note, setNote] = useState("");

  const [isCreating, setIsCreating] = useState(false);

  useEffect(() => {
    dialogRef.current?.showModal();
  }, []);

  async function handleSubmit(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    const trimmedTitle = title.trim();

    if (trimmedTitle === "") {
      showToast("Title is required");
      return;
    }

    setIsCreating(true);

    await createCustomShoppingEntry(householdId, {
      title: trimmedTitle,
      quantity,
      priority,
      note: note.trim() === "" ? null : note.trim(),
    })
      .then(async () => {
        await onCreated();
        dialogRef.current?.close();
      })
      .catch(() => showToast("Failed to create shopping entry", "error"))
      .finally(() => setIsCreating(false));
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
            <h2>Add shopping item</h2>
            <p>Add something to your shopping list</p>
          </div>

          <button
            type="button"
            className="button button--ghost inventory-item-dialog__close"
            onClick={() => dialogRef.current?.close()}
          >
            Close
          </button>
        </header>

        <form
          className="inventory-item-dialog__section"
          onSubmit={handleSubmit}
        >
          <ShoppingEntryFields
            title={title}
            quantity={quantity}
            priority={priority}
            note={note}
            onTitleChange={setTitle}
            onQuantityChange={setQuantity}
            onPriorityChange={setPriority}
            onNoteChange={setNote}
            disabled={isCreating}
          />

          <button
            type="submit"
            className="button button--primary"
            disabled={isCreating}
          >
            {isCreating ? "Adding..." : "Add item"}
          </button>
        </form>
      </div>
    </dialog>
  );
}
