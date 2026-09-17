import { useState } from "react";

import { useToast } from "../../../../components/toast/ToastContext";
import { restoreInventoryItem } from "../../api";
import type { InventoryItem } from "../../types";

import styles from "./ArchivedInventoryItemRow.module.css";

type Props = {
  householdId: string;
  item: InventoryItem;
  onChanged: () => Promise<void>;
};

export function ArchivedInventoryItemRow({
  householdId,
  item,
  onChanged,
}: Props) {
  const { showToast } = useToast();

  const [isRestoring, setIsRestoring] = useState(false);

  function handleRestore() {
    setIsRestoring(true);

    void restoreInventoryItem(householdId, item.id)
      .then(async () => {
        await onChanged();
        showToast("Item restored", "success");
      })
      .catch(() => showToast("Failed to restore item", "error"))
      .finally(() => setIsRestoring(false));
  }

  return (
    <div className={styles.row}>
      <div className={styles.info}>
        <strong>{item.name}</strong>

        <span>{item.category?.name ?? "No category"}</span>
      </div>

      <button
        type="button"
        className="button button--secondary"
        onClick={handleRestore}
        disabled={isRestoring}
      >
        {isRestoring ? "Restoring..." : "Restore"}
      </button>
    </div>
  );
}
