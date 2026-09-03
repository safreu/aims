import { useEffect, useRef, useState, type SubmitEvent } from "react";
import { useToast } from "../../../components/toast/ToastContext";
import { createHousehold } from "../api";

type CreateHouseholdDialogProps = {
  onCreated: () => Promise<void>;
  onClose: () => void;
};

export function CreateHouseholdDialog({
  onCreated,
  onClose,
}: CreateHouseholdDialogProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);

  const [name, setName] = useState("");
  const [kind, setKind] = useState<"shared" | "personal">("shared");

  const [isCreating, setIsCreating] = useState(false);
  const [validationError, setValidationError] = useState<string | null>(null);

  const { showToast } = useToast();

  useEffect(() => {
    dialogRef.current?.showModal();
  }, []);

  function handleSubmit(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    if (name.trim() === "") {
      setValidationError("Name is required");
      return;
    }

    setValidationError(null);
    setIsCreating(true);

    void createHousehold({
      name: name.trim(),
      kind: kind,
    })
      .then(async () => {
        await onCreated();
        showToast("Household created", "success");
        dialogRef.current?.close();
      })
      .catch(() => showToast("Failed to create household", "error"))
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
      <form className="inventory-item-dialog__content" onSubmit={handleSubmit}>
        <header className="inventory-item-dialog__header">
          <div>
            <h2>Create Household</h2>
            <p>Create a new household for your inventory and shopping list</p>
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

        <section className="inventory-item-dialog__section">
          <div className="inventory-item-dialog__fields">
            <label className="inventory-item-dialog__field">
              <span>Name</span>

              <input
                type="text"
                value={name}
                onChange={(event) => setName(event.target.value)}
                disabled={isCreating}
                autoFocus
              />
            </label>

            <label className="inventory-item-dialog__field">
              <span>Kind</span>

              <select
                value={kind}
                onChange={(event) =>
                  setKind(event.target.value as "personal" | "shared")
                }
                disabled={isCreating}
              >
                <option value="shared">Shared</option>
                <option value="personal">Personal</option>
              </select>
            </label>
          </div>

          {validationError !== null && (
            <p className="inventory-item-dialog__error">{validationError}</p>
          )}
        </section>

        <footer className="inventory-item-dialog__actions">
          <button
            type="button"
            className="button button--secondary"
            onClick={() => dialogRef.current?.close()}
            disabled={isCreating}
          >
            Cancel
          </button>

          <button
            type="submit"
            className="button button--primary"
            disabled={isCreating}
          >
            {isCreating ? "Creating..." : "Create household"}
          </button>
        </footer>
      </form>
    </dialog>
  );
}
