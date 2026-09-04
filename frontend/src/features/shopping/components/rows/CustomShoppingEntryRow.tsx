import { useState } from "react";
import type { CustomShoppingEntry } from "../../types";
import { setCustomShoppingChecked } from "../../api";
import { CustomShoppingEntryDialog } from "../dialogs/CustomShoppingEntryDialog";
import { PriorityIndicator } from "../../../inventory/components/priority/PriorityIndicator";

type CustomShoppingEntryRowProps = {
  householdId: string;
  entry: CustomShoppingEntry;
  onChange: () => Promise<void>;
};

export function CustomShoppingEntryRow({
  householdId,
  entry,
  onChange,
}: CustomShoppingEntryRowProps) {
  const [isMutating, setIsMutating] = useState(false);
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleCheckedUpdate(checked: boolean) {
    setIsMutating(true);
    setError(null);

    await setCustomShoppingChecked(householdId, entry.id, { checked })
      .then(() => onChange())
      .catch(() => setError("Failed to update shopping list"))
      .finally(() => setIsMutating(false));
  }

  return (
    <li
      className={`shopping-entry ${entry.checked ? "shopping-entry--checked" : ""}`}
    >
      <input
        className="shopping-entry__checkbox"
        type="checkbox"
        checked={entry.checked}
        disabled={isMutating}
        aria-label={`Mark ${entry.title} as bought`}
        onChange={(event) => void handleCheckedUpdate(event.target.checked)}
      />

      <button
        type="button"
        className="shopping-entry__open"
        onClick={() => setIsDialogOpen(true)}
      >
        <div className="shopping-entry__main">
          <div className="shopping-entry__title">
            <strong className="shopping-entry__name">{entry.title}</strong>

            <PriorityIndicator priority={entry.priority} />
          </div>

          <strong className="shopping-entry__quantity">
            ×{entry.quantity}
          </strong>
        </div>

        {entry.note !== null && (
          <p className="shopping-entry__note">{entry.note}</p>
        )}
      </button>

      {error !== null && <p className="shopping-entry__error">{error}</p>}

      {isDialogOpen && (
        <CustomShoppingEntryDialog
          householdId={householdId}
          entry={entry}
          onChanged={onChange}
          onClose={() => setIsDialogOpen(false)}
        />
      )}
    </li>
  );
}
