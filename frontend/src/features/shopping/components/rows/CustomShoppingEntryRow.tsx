import { useState } from "react";

import { useToast } from "../../../../components/toast/ToastContext";
import { PriorityIndicator } from "../../../../components/priority/PriorityIndicator";
import { setCustomShoppingChecked } from "../../api";
import type { CustomShoppingEntry } from "../../types";
import { CustomShoppingEntryDialog } from "../dialogs/CustomShoppingEntryDialog";

import styles from "./ShoppingEntryRow.module.css";
import { SwipeActions } from "../../../../components/actions/SwipeActions";

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
  const { showToast } = useToast();

  const [isMutating, setIsMutating] = useState(false);
  const [isDialogOpen, setIsDialogOpen] = useState(false);

  async function handleCheckedUpdate(checked: boolean) {
    setIsMutating(true);

    try {
      await setCustomShoppingChecked(householdId, entry.id, { checked });

      await onChange();
    } catch {
      showToast("Failed to update shopping list", "error");
    } finally {
      setIsMutating(false);
    }
  }

  return (
    <SwipeActions
      as="li"
      disabled={isMutating}
      onSwipeRight={() => handleCheckedUpdate(!entry.checked)}
      rightLabel={entry.checked ? "Uncheck" : "Check"}
      rightVariant="success"
    >
      <li className={`${styles.entry} ${entry.checked ? styles.checked : ""}`}>
        <input
          className={styles.checkbox}
          type="checkbox"
          checked={entry.checked}
          disabled={isMutating}
          aria-label={`Mark ${entry.title} as bought`}
          onChange={(event) => void handleCheckedUpdate(event.target.checked)}
        />

        <button
          type="button"
          className={styles.open}
          onClick={() => setIsDialogOpen(true)}
        >
          <div className={styles.main}>
            <div className={styles.title}>
              <strong className={styles.name}>{entry.title}</strong>

              <PriorityIndicator priority={entry.priority} />
            </div>

            <strong className={styles.quantity}>×{entry.quantity}</strong>
          </div>

          {entry.note !== null && <p className={styles.note}>{entry.note}</p>}
        </button>

        {isDialogOpen && (
          <CustomShoppingEntryDialog
            householdId={householdId}
            entry={entry}
            onChanged={onChange}
            onClose={() => setIsDialogOpen(false)}
          />
        )}
      </li>
    </SwipeActions>
  );
}
