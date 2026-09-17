import { useEffect, useRef, useState, type SubmitEvent } from "react";

import { Select, type SelectOption } from "../../../components/select/Select";
import { useToast } from "../../../components/toast/ToastContext";
import { createHousehold } from "../api";
import type { Household } from "../types";

type CreateHouseholdDialogProps = {
  onCreated: () => Promise<void>;
  onClose: () => void;
};

const householdKindOptions: SelectOption<Household["kind"]>[] = [
  {
    value: "shared",
    label: "Shared",
  },
  {
    value: "personal",
    label: "Personal",
  },
];

export function CreateHouseholdDialog({
  onCreated,
  onClose,
}: CreateHouseholdDialogProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);

  const [name, setName] = useState("");
  const [kind, setKind] = useState<Household["kind"]>("shared");

  const [isCreating, setIsCreating] = useState(false);
  const [validationError, setValidationError] = useState<string | null>(null);

  const { showToast } = useToast();

  useEffect(() => {
    dialogRef.current?.showModal();
  }, []);

  async function handleSubmit(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    const trimmedName = name.trim();

    if (trimmedName === "") {
      setValidationError("Name is required");
      return;
    }

    setValidationError(null);
    setIsCreating(true);

    try {
      await createHousehold({
        name: trimmedName,
        kind,
      });

      await onCreated();

      showToast("Household created", "success");

      dialogRef.current?.close();
    } catch {
      showToast("Failed to create household", "error");
    } finally {
      setIsCreating(false);
    }
  }

  return (
    <dialog
      ref={dialogRef}
      className="dialog"
      onClose={onClose}
      onClick={(event) => {
        if (event.target === dialogRef.current && !isCreating) {
          dialogRef.current?.close();
        }
      }}
    >
      <form className="dialog__content" onSubmit={handleSubmit}>
        <div className="dialog__header">
          <h2 className="dialog__title">Create household</h2>

          <p className="dialog__description">
            Create a new household for your inventory and shopping list.
          </p>
        </div>

        <div className="dialog__field">
          <label htmlFor="household-name">Name</label>

          <input
            id="household-name"
            type="text"
            value={name}
            onChange={(event) => {
              setName(event.target.value);

              if (validationError !== null) {
                setValidationError(null);
              }
            }}
            disabled={isCreating}
            autoFocus
          />
        </div>

        <div className="dialog__field">
          <label>Kind</label>

          <Select
            value={kind}
            options={householdKindOptions}
            onValueChange={(value) => setKind(value as "personal" | "shared")}
            disabled={isCreating}
            ariaLabel="Household kind"
            portal={false}
          />
        </div>

        {validationError !== null && (
          <p className="form-error" role="alert">
            {validationError}
          </p>
        )}

        <div className="dialog__actions">
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
            disabled={isCreating || name.trim() === ""}
          >
            {isCreating ? "Creating..." : "Create household"}
          </button>
        </div>
      </form>
    </dialog>
  );
}
