import { useState } from "react";
import type { InventoryShoppingEntry } from "../../types";
import { setShoppingChecked } from "../../api";
import "./ShoppingEntryRow.css";
import { InventoryShoppingEntryDialog } from "../dialogs/InventoryShoppingEntryDialog";

type InventoryShoppingEntryRowProps = {
  householdId: string;
  entry: InventoryShoppingEntry;
  onChange: () => Promise<void>;
};

export function InventoryShoppingEntryRow({
  householdId,
  entry,
  onChange,
}: InventoryShoppingEntryRowProps) {
  const [isDialogOpen, setIsDialogOpen] = useState(false);

  const [isMutating, setIsMutating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleCheckedUpdate(checked: boolean) {
    setIsMutating(true);
    setError(null);

    await setShoppingChecked(householdId, entry.item_id, { checked })
      .then(() => onChange())
      .catch(() => setError("Failed ti update shopping list"))
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
        aria-label={`Mark ${entry.name} as bought`}
        onChange={(event) => void handleCheckedUpdate(event.target.checked)}
      />

      <button
        type="button"
        className="shopping-entry__open"
        onClick={() => setIsDialogOpen(true)}
      >
        <div className="shopping-entry__main">
          <strong className="shopping-entry__name">{entry.name}</strong>

          <strong className="shopping-entry__quantity">
            ×{entry.quantity}
          </strong>
        </div>

        <div className="shopping-entry__meta">
          {entry.category !== null && <span>{entry.category.name}</span>}

          {entry.category !== null && <span aria-hidden="true">·</span>}

          <span>{formatPriority(entry.priority)}</span>
        </div>

        {entry.note !== null && (
          <p className="shopping-entry__note">{entry.note}</p>
        )}
      </button>

      {error !== null && <p className="shopping-entry__error">{error}</p>}

      {isDialogOpen && (
        <InventoryShoppingEntryDialog
          householdId={householdId}
          entry={entry}
          onChanged={onChange}
          onClose={() => setIsDialogOpen(false)}
        />
      )}
    </li>
  );
}

function formatPriority(priority: string) {
  if (priority === "default") {
    return "Normal";
  }

  return priority.charAt(0).toUpperCase() + priority.slice(1);
}
