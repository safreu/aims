import { useEffect, useRef, useState, type SubmitEvent } from "react";
import { createInventoryCategory } from "../../api";
import "./InventoryItemDialog.css";
import { useToast } from "../../../../components/toast/ToastContext";

type CreateInventoryCategoryDialogProps = {
  householdId: string;
  onCreated: (categoryId: string) => Promise<void>;
  onClose: () => void;
};

export function CreateInventoryCategoryDialog({
  householdId,
  onCreated,
  onClose,
}: CreateInventoryCategoryDialogProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const { showToast } = useToast();

  const [name, setName] = useState("");

  const [isCreating, setIsCreating] = useState(false);

  useEffect(() => {
    dialogRef.current?.showModal();
  }, []);

  async function handleSubmit(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();
    event.stopPropagation();

    setIsCreating(true);

    await createInventoryCategory(householdId, { name })
      .then(async (category) => {
        await onCreated(category.id);
        dialogRef.current?.close();
        showToast("Category creation successful", "success");
      })
      .catch(() => showToast("Failed to create inventory category", "error"))
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
            <h2>Add Category</h2>
            <p>Create a new inventory category</p>
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
          <div className="inventory-item-dialog__fields inventory-item-dialog__fields--single">
            <label className="inventory-item-dialog__field">
              <span>Name</span>
              <input
                type="text"
                value={name}
                onChange={(event) => setName(event.target.value)}
                required
              />
            </label>
          </div>

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
