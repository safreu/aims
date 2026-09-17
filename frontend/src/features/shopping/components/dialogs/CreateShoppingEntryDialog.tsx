import { X } from "lucide-react";
import { useEffect, useRef, useState, type SubmitEvent } from "react";

import { useToast } from "../../../../components/toast/ToastContext";
import type { Priority } from "../../../../domain/priority";
import { createCustomShoppingEntry } from "../../api";
import { ShoppingEntryFields } from "../fields/ShoppingEntryFields";

type ShoppingEntryFieldErrors = {
  title?: string;
  quantity?: string;
};

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
  const [quantity, setQuantity] = useState<number | "">(1);
  const [priority, setPriority] = useState<Priority>("default");
  const [note, setNote] = useState("");

  const [isCreating, setIsCreating] = useState(false);
  const [fieldErrors, setFieldErrors] = useState<ShoppingEntryFieldErrors>({});

  useEffect(() => {
    dialogRef.current?.showModal();
  }, []);

  async function handleSubmit(event: SubmitEvent<HTMLFormElement>) {
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

    setIsCreating(true);

    try {
      await createCustomShoppingEntry(householdId, {
        title: title.trim(),
        quantity,
        priority,
        note: note.trim() === "" ? null : note.trim(),
      });

      await onCreated();

      dialogRef.current?.close();
      showToast("Shopping item added", "success");
    } catch {
      showToast("Failed to create shopping item", "error");
    } finally {
      setIsCreating(false);
    }
  }

  function handleBackdropClick(event: React.MouseEvent<HTMLDialogElement>) {
    if (event.target === dialogRef.current && !isCreating) {
      dialogRef.current?.close();
    }
  }

  return (
    <dialog
      ref={dialogRef}
      className="dialog"
      onClose={onClose}
      onCancel={(event) => {
        if (isCreating) {
          event.preventDefault();
        }
      }}
      onClick={handleBackdropClick}
    >
      <div className="dialog__content">
        <header className="dialog__header">
          <div>
            <h2 className="dialog__title">Add shopping item</h2>

            <p className="dialog__description">
              Add something to your shopping list
            </p>
          </div>

          <button
            type="button"
            className="dialog__close"
            onClick={() => dialogRef.current?.close()}
            disabled={isCreating}
            aria-label="Close"
          >
            <X aria-hidden="true" />
          </button>
        </header>

        <form className="dialog__section" onSubmit={handleSubmit}>
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
            disabled={isCreating}
          />

          <div className="dialog__actions">
            <button
              type="submit"
              className="button button--primary"
              disabled={isCreating}
            >
              {isCreating ? "Adding..." : "Add item"}
            </button>
          </div>
        </form>
      </div>
    </dialog>
  );
}
