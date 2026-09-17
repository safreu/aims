import { useState, type SubmitEvent } from "react";

import { Dialog } from "../../../../components/dialog/Dialog";
import { useToast } from "../../../../components/toast/ToastContext";
import type { Priority } from "../../../../domain/priority";
import { createInventoryItem } from "../../api";
import { InventoryItemFields } from "../fields/InventoryItemFields";

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
  const { showToast } = useToast();

  const [name, setName] = useState("");
  const [categoryId, setCategoryId] = useState<string | null>(null);
  const [currentStock, setCurrentStock] = useState<number | "">(0);
  const [reorderThreshold, setReorderThreshold] = useState<number | "">(0);
  const [priority, setPriority] = useState<Priority>("default");

  const [isCreating, setIsCreating] = useState(false);
  const [fieldErrors, setFieldErrors] = useState<InventoryItemFieldErrors>({});

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

    try {
      await createInventoryItem(householdId, {
        name: name.trim(),
        category_id: categoryId,
        current_stock: currentStock,
        reorder_threshold: reorderThreshold,
        priority,
      });

      await onCreated();

      onClose();
      showToast("Item created", "success");
    } catch {
      showToast("Failed to create inventory item", "error");
    } finally {
      setIsCreating(false);
    }
  }

  return (
    <Dialog
      title="Add item"
      description="Create a new inventory item"
      onClose={onClose}
      closeDisabled={isCreating}
    >
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

                if (fieldErrors.currentStock !== undefined) {
                  setFieldErrors((current) => ({
                    ...current,
                    currentStock: undefined,
                  }));
                }
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
    </Dialog>
  );
}
