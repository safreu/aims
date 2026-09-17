import { useState } from "react";
import { Check, Trash2 } from "lucide-react";

import { SwipeActions } from "../../../../components/actions/SwipeActions";
import { PriorityIndicator } from "../../../../components/priority/PriorityIndicator";
import { useToast } from "../../../../components/toast/ToastContext";
import { deleteCustomShoppingEntry, setCustomShoppingChecked } from "../../api";
import type { CustomShoppingEntry } from "../../types";
import { CustomShoppingEntryDialog } from "../dialogs/CustomShoppingEntryDialog";

import styles from "./ShoppingEntryRow.module.css";

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

  async function handleDelete() {
    setIsMutating(true);

    try {
      await deleteCustomShoppingEntry(householdId, entry.id);

      await onChange();

      showToast("Shopping item deleted", "success");
    } catch {
      showToast("Failed to delete shopping item", "error");
    } finally {
      setIsMutating(false);
    }
  }

  return (
    <SwipeActions
      as="li"
      disabled={isMutating}
      leftIcon={<Trash2 aria-hidden="true" />}
      rightIcon={<Check aria-hidden="true" />}
      leftVariant="danger"
      rightVariant="success"
      onSwipeLeft={handleDelete}
      onSwipeRight={() => handleCheckedUpdate(!entry.checked)}
    >
      <div className={`${styles.entry} ${entry.checked ? styles.checked : ""}`}>
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
      </div>
    </SwipeActions>
  );
}
