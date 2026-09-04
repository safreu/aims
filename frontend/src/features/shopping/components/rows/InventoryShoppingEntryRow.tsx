import { useState } from "react";
import type { InventoryShoppingEntry } from "../../types";
import { setShoppingChecked } from "../../api";
import "./ShoppingEntryRow.css";
import { InventoryShoppingEntryDialog } from "../dialogs/InventoryShoppingEntryDialog";
import { PriorityIndicator } from "../../../inventory/components/priority/PriorityIndicator";
import { useToast } from "../../../../components/toast/ToastContext";

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
  const { showToast } = useToast();

  const [isDialogOpen, setIsDialogOpen] = useState(false);

  const [isMutating, setIsMutating] = useState(false);

  async function handleCheckedUpdate(checked: boolean) {
    setIsMutating(true);

    await setShoppingChecked(householdId, entry.item_id, { checked })
      .then(() => onChange())
      .catch(() => showToast("Failed to update shopping list", "error"))
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
          <div className="shopping-entry__title">
            <strong className="shopping-entry__name">{entry.name}</strong>

            <PriorityIndicator priority={entry.priority} />
          </div>

          <strong className="shopping-entry__quantity">
            ×{entry.quantity}
          </strong>
        </div>

        <div className="shopping-entry__meta">
          {entry.category !== null && (
            <span className="shopping-entry__category">
              {entry.category.name}
            </span>
          )}
        </div>

        {entry.note !== null && (
          <p className="shopping-entry__note">{entry.note}</p>
        )}
      </button>

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
