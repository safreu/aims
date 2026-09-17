import { useEffect, useRef, useState, type SubmitEvent } from "react";

import { useToast } from "../../../../components/toast/ToastContext";
import { createInventoryCategory } from "../../api";

type Props = {
  householdId: string;
  onCreated: (categoryId: string) => Promise<void>;
  onClose: () => void;
};

export function CreateInventoryCategoryDialog({
  householdId,
  onCreated,
  onClose,
}: Props) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const { showToast } = useToast();

  const [name, setName] = useState("");
  const [nameError, setNameError] = useState<string>();
  const [isCreating, setIsCreating] = useState(false);

  useEffect(() => {
    dialogRef.current?.showModal();
  }, []);

  async function handleSubmit(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();
    event.stopPropagation();

    const trimmedName = name.trim();

    if (trimmedName === "") {
      setNameError("Name is required");
      return;
    }

    setNameError(undefined);
    setIsCreating(true);

    await createInventoryCategory(householdId, {
      name: trimmedName,
    })
      .then(async (category) => {
        await onCreated(category.id);
        dialogRef.current?.close();
        showToast("Category created", "success");
      })
      .catch(() => showToast("Failed to create inventory category", "error"))
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
            <h2 className="dialog__title">Add category</h2>
            <p className="dialog__description">
              Create a new inventory category
            </p>
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
          <div className="dialog__fields dialog__fields--single">
            <label
              className={`dialog__field ${
                nameError ? "dialog__field--error" : ""
              }`}
            >
              <span>Name</span>

              <input
                type="text"
                value={name}
                onChange={(event) => {
                  setName(event.target.value);

                  if (nameError !== undefined) {
                    setNameError(undefined);
                  }
                }}
                disabled={isCreating}
              />

              {nameError && (
                <span className="dialog__field-error" role="alert">
                  {nameError}
                </span>
              )}
            </label>
          </div>

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
