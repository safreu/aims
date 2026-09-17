import { useEffect, useRef, useState, type SubmitEvent } from "react";

import { useToast } from "../../../../components/toast/ToastContext";
import {
  deleteCustomShoppingEntry,
  updateCustomShoppingEntry,
} from "../../api";
import type { CustomShoppingEntry } from "../../types";
import type { Priority } from "../../../../domain/priority";
import { ShoppingEntryFields } from "../fields/ShoppingEntryFields";

type ShoppingEntryFieldErrors = {
  title?: string;
  quantity?: string;
};

type CustomShoppingEntryDialogProps = {
  householdId: string;
  entry: CustomShoppingEntry;
  onChanged: () => Promise<void>;
  onClose: () => void;
};

export function CustomShoppingEntryDialog({
  householdId,
  entry,
  onChanged,
  onClose,
}: CustomShoppingEntryDialogProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const { showToast } = useToast();

  const [title, setTitle] = useState(entry.title);
  const [quantity, setQuantity] = useState<number | "">(entry.quantity);
  const [priority, setPriority] = useState<Priority>(entry.priority);
  const [note, setNote] = useState(entry.note ?? "");

  const [isMutating, setIsMutating] = useState(false);
  const [fieldErrors, setFieldErrors] = useState<ShoppingEntryFieldErrors>({});

  useEffect(() => {
    dialogRef.current?.showModal();
  }, []);

  async function handleSave(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    const errors: ShoppingEntryFieldErrors = {};

    if (title.trim() === "") {
      errors.title = "Name is required";
    }

    if (quantity === "") {
      errors.quantity = "Quantity is required";
    } else if (!Number.isInteger(quantity) || quantity < 1) {
      errors.quantity = "Quantity must be a positive whole number";
    }

    setFieldErrors(errors);

    if (Object.keys(errors).length > 0 || quantity === "") {
      return;
    }

    setIsMutating(true);

    try {
      await updateCustomShoppingEntry(householdId, entry.id, {
        title: title.trim(),
        quantity,
        priority,
        note: note.trim() === "" ? null : note.trim(),
      });

      await onChanged();

      dialogRef.current?.close();
      showToast("Shopping item updated", "success");
    } catch {
      showToast("Failed to update shopping item", "error");
    } finally {
      setIsMutating(false);
    }
  }

  async function handleDelete() {
    setIsMutating(true);

    try {
      await deleteCustomShoppingEntry(householdId, entry.id);

      await onChanged();

      dialogRef.current?.close();
      showToast("Shopping item deleted", "success");
    } catch {
      showToast("Failed to delete shopping item", "error");
    } finally {
      setIsMutating(false);
    }
  }

  function handleBackdropClick(event: React.MouseEvent<HTMLDialogElement>) {
    if (event.target === dialogRef.current && !isMutating) {
      dialogRef.current?.close();
    }
  }

  return (
    <dialog
      ref={dialogRef}
      className="dialog"
      onClose={onClose}
      onCancel={(event) => {
        if (isMutating) {
          event.preventDefault();
        }
      }}
      onClick={handleBackdropClick}
    >
      <div className="dialog__content">
        <header className="dialog__header">
          <div>
            <h2 className="dialog__title">{entry.title}</h2>

            <p className="dialog__description">Edit this shopping item</p>
          </div>

          <button
            type="button"
            className="button button--ghost"
            onClick={() => dialogRef.current?.close()}
            disabled={isMutating}
          >
            Close
          </button>
        </header>

        <form className="dialog__section" onSubmit={handleSave}>
          <ShoppingEntryFields
            title={title}
            quantity={quantity}
            priority={priority}
            note={note}
            onTitleChange={setTitle}
            onQuantityChange={setQuantity}
            onPriorityChange={setPriority}
            onNoteChange={setNote}
            titleError={fieldErrors.title}
            quantityError={fieldErrors.quantity}
            disabled={isMutating}
          />

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
          <h3>Delete item</h3>

          <p>Permanently remove this custom item from the shopping list</p>

          <div className="dialog__actions">
            <button
              type="button"
              className="button button--danger"
              disabled={isMutating}
              onClick={() => void handleDelete()}
            >
              Delete
            </button>
          </div>
        </section>
      </div>
    </dialog>
  );
}
