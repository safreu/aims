import { useEffect, useRef, useState, type SubmitEvent } from "react";
import { type InventoryItemPriority } from "../../types";
import { createInventoryItem } from "../../api";
import "./InventoryItemDialog.css";
import { InventoryItemFields } from "../fields/InventoryItemFields";
import { useToast } from "../../../../components/toast/ToastContext";

type CreateInventoryItemDialogProps = {
  householdId: string;
  onCreated: () => Promise<void>;
  onClose: () => void;
};

export function CreateInventoryItemDialog({
  householdId,
  onCreated,
  onClose,
}: CreateInventoryItemDialogProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const { showToast } = useToast();

  const [name, setName] = useState("");
  const [categoryId, setCategoryId] = useState<string | null>(null);
  const [currentStock, setCurrentStock] = useState<number | "">(0);
  const [reorderThreshold, setReorderThreshold] = useState<number | "">(0);
  const [priority, setPriority] = useState<InventoryItemPriority>("default");

  const [isCreating, setIsCreating] = useState(false);

  useEffect(() => {
    dialogRef.current?.showModal();
  }, []);

  async function handleSubmit(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    if (reorderThreshold === "" || currentStock === "") {
      showToast("Values must not be empty", "warning");
      return;
    }

    setIsCreating(true);

    await createInventoryItem(householdId, {
      name,
      category_id: categoryId,
      current_stock: currentStock,
      reorder_threshold: reorderThreshold,
      priority,
    })
      .then(async () => {
        await onCreated();
        dialogRef.current?.close();
        showToast("Item creation successful", "success");
      })
      .catch(() => showToast("Failed to create inventory item", "error"))
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
            <h2>Add item</h2>
            <p>Create a new inventory item</p>
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
            disabled={isCreating}
          >
            <label className="inventory-item-dialog__field">
              <span>Current stock</span>

              <input
                type="number"
                min="0"
                value={currentStock}
                onChange={(event) => {
                  const value = event.target.value;
                  setCurrentStock(value === "" ? "" : Number(value));
                }}
                disabled={isCreating}
                required
              />
            </label>
          </InventoryItemFields>

          <button
            type="submit"
            className="button button--primary"
            disabled={isCreating}
          >
            {isCreating ? "Creating..." : "Create"}
          </button>
        </form>
      </div>
    </dialog>
  );
}
