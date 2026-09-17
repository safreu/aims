import { useState, type SubmitEvent } from "react";

import { Dialog } from "../../../components/dialog/Dialog";
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
  const [name, setName] = useState("");
  const [kind, setKind] = useState<Household["kind"]>("shared");

  const [isCreating, setIsCreating] = useState(false);
  const [validationError, setValidationError] = useState<string | null>(null);

  const { showToast } = useToast();

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

      onClose();
    } catch {
      showToast("Failed to create household", "error");
    } finally {
      setIsCreating(false);
    }
  }

  return (
    <Dialog
      title="Create household"
      description="Create a new household for your inventory and shopping list."
      onClose={onClose}
      closeDisabled={isCreating}
    >
      <form className="dialog__form" onSubmit={handleSubmit}>
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
            onValueChange={(value) => setKind(value as Household["kind"])}
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
            onClick={onClose}
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
    </Dialog>
  );
}
