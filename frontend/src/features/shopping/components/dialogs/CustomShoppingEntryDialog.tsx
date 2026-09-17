import { useState, type SubmitEvent } from "react";

import { Dialog } from "../../../../components/dialog/Dialog";
import { useToast } from "../../../../components/toast/ToastContext";
import type { Priority } from "../../../../domain/priority";
import {
  deleteCustomShoppingEntry,
  updateCustomShoppingEntry,
} from "../../api";
import type { CustomShoppingEntry } from "../../types";
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
  const { showToast } = useToast();

  const [title, setTitle] = useState(entry.title);
  const [quantity, setQuantity] = useState<number | "">(entry.quantity);
  const [priority, setPriority] = useState<Priority>(entry.priority);
  const [note, setNote] = useState(entry.note ?? "");

  const [isMutating, setIsMutating] = useState(false);
  const [fieldErrors, setFieldErrors] = useState<ShoppingEntryFieldErrors>({});

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

      onClose();
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

      onClose();
      showToast("Shopping item deleted", "success");
    } catch {
      showToast("Failed to delete shopping item", "error");
    } finally {
      setIsMutating(false);
    }
  }

  return (
    <Dialog
      title={entry.title}
      description="Edit this shopping item"
      onClose={onClose}
      closeDisabled={isMutating}
    >
      <form className="dialog__section" onSubmit={handleSave}>
        <ShoppingEntryFields
          title={title}
          quantity={quantity}
          priority={priority}
          note={note}
          onTitleChange={(value) => {
            setTitle(value);

            if (fieldErrors.title !== undefined) {
              setFieldErrors((current) => ({
                ...current,
                title: undefined,
              }));
            }
          }}
          onQuantityChange={(value) => {
            setQuantity(value);

            if (fieldErrors.quantity !== undefined) {
              setFieldErrors((current) => ({
                ...current,
                quantity: undefined,
              }));
            }
          }}
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
    </Dialog>
  );
}
