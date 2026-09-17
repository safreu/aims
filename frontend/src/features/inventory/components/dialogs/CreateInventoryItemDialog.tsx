import { useEffect, useRef, useState, type SubmitEvent } from "react";

import { useToast } from "../../../../components/toast/ToastContext";
import { createInventoryItem } from "../../api";
import { InventoryItemFields } from "../fields/InventoryItemFields";
import type { Priority } from "../../../../domain/priority";

type InventoryItemFieldErrors = {
  name?: string;
  currentStock?: string;
  reorderThreshold?: string;
};

type Props = {
  householdId: string;
  onCreated: () => Promise<void>;
  onClose: () => void;
};

export function CreateInventoryItemDialog({
  householdId,
  onCreated,
  onClose,
}: Props) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const { showToast } = useToast();

  const [name, setName] = useState("");
  const [categoryId, setCategoryId] = useState<string | null>(null);
  const [currentStock, setCurrentStock] = useState<number | "">(0);
  const [reorderThreshold, setReorderThreshold] = useState<number | "">(0);
  const [priority, setPriority] = useState<Priority>("default");

  const [isCreating, setIsCreating] = useState(false);
  const [fieldErrors, setFieldErrors] = useState<InventoryItemFieldErrors>({});

  useEffect(() => {
    dialogRef.current?.showModal();
  }, []);

  async function handleSubmit(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    const errors: InventoryItemFieldErrors = {};

    if (name.trim() === "") {
      errors.name = "Name is required";
    }

    if (currentStock === "") {
      errors.currentStock = "Current stock is required";
    } else if (!Number.isInteger(currentStock) || currentStock < 0) {
      errors.currentStock = "Current stock must be a non-negative whole number";
    }

    if (reorderThreshold === "") {
      errors.reorderThreshold = "Reorder threshold is required";
    } else if (!Number.isInteger(reorderThreshold) || reorderThreshold < 0) {
      errors.reorderThreshold =
        "Reorder threshold must be a non-negative whole number";
    }

    setFieldErrors(errors);

    if (
      Object.keys(errors).length > 0 ||
      currentStock === "" ||
      reorderThreshold === ""
    ) {
      return;
    }

    setIsCreating(true);

    await createInventoryItem(householdId, {
      name: name.trim(),
      category_id: categoryId,
      current_stock: currentStock,
      reorder_threshold: reorderThreshold,
      priority,
    })
      .then(async () => {
        await onCreated();
        dialogRef.current?.close();
        showToast("Item created", "success");
      })
      .catch(() => showToast("Failed to create inventory item", "error"))
      .finally(() => setIsCreating(false));
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
            <h2 className="dialog__title">Add item</h2>
            <p className="dialog__description">Create a new inventory item</p>
          </div>

          <button
            type="button"
            className="button button--ghost"
            onClick={() => dialogRef.current?.close()}
            disabled={isCreating}
          >
            Close
          </button>
        </header>

        <form className="dialog__section" onSubmit={handleSubmit}>
          <InventoryItemFields
            householdId={householdId}
            name={name}
            categoryId={categoryId}
            reorderThreshold={reorderThreshold}
            priority={priority}
            onNameChange={setName}
            onCategoryChange={setCategoryId}
            onReorderThresholdChange={setReorderThreshold}
            onPriorityChange={setPriority}
            nameError={fieldErrors.name}
            reorderThresholdError={fieldErrors.reorderThreshold}
            disabled={isCreating}
          >
            <label
              className={`dialog__field ${
                fieldErrors.currentStock ? "dialog__field--error" : ""
              }`}
            >
              <span>Current stock</span>

              <input
                type="number"
                min="0"
                step="1"
                value={currentStock}
                onChange={(event) => {
                  const value = event.target.value;

                  setCurrentStock(value === "" ? "" : Number(value));
                }}
                disabled={isCreating}
              />

              {fieldErrors.currentStock && (
                <span className="dialog__field-error" role="alert">
                  {fieldErrors.currentStock}
                </span>
              )}
            </label>
          </InventoryItemFields>

          <div className="dialog__actions">
            <button
              type="submit"
              className="button button--primary"
              disabled={isCreating}
            >
              {isCreating ? "Creating..." : "Create"}
            </button>
          </div>
        </form>
      </div>
    </dialog>
  );
}
