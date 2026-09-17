import { useState } from "react";

import { useToast } from "../../../../components/toast/ToastContext";
import { PriorityIndicator } from "../../../../components/priority/PriorityIndicator";
import { setShoppingChecked } from "../../api";
import type { InventoryShoppingEntry } from "../../types";
import { InventoryShoppingEntryDialog } from "../dialogs/InventoryShoppingEntryDialog";

import styles from "./ShoppingEntryRow.module.css";

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

  const [isMutating, setIsMutating] = useState(false);
  const [isDialogOpen, setIsDialogOpen] = useState(false);

  async function handleCheckedUpdate(checked: boolean) {
    setIsMutating(true);

    try {
      await setShoppingChecked(householdId, entry.item_id, { checked });

      await onChange();
    } catch {
      showToast("Failed to update shopping list", "error");
    } finally {
      setIsMutating(false);
    }
  }

  return (
    <li className={`${styles.entry} ${entry.checked ? styles.checked : ""}`}>
      <input
        className={styles.checkbox}
        type="checkbox"
        checked={entry.checked}
        disabled={isMutating}
        aria-label={`Mark ${entry.name} as bought`}
        onChange={(event) => void handleCheckedUpdate(event.target.checked)}
      />

      <button
        type="button"
        className={styles.open}
        onClick={() => setIsDialogOpen(true)}
      >
        <div className={styles.main}>
          <div className={styles.title}>
            <strong className={styles.name}>{entry.name}</strong>

            <PriorityIndicator priority={entry.priority} />
          </div>

          <strong className={styles.quantity}>×{entry.quantity}</strong>
        </div>

        {entry.category !== null && (
          <div className={styles.meta}>
            <span className={styles.category}>{entry.category.name}</span>
          </div>
        )}

        {entry.note !== null && <p className={styles.note}>{entry.note}</p>}
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
