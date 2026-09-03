import { useState } from "react";
import { restoreInventoryItem } from "../../api";
import type { InventoryItem } from "../../types";
import "./ArchivedInventoryItemRow.css";
import { useToast } from "../../../../components/toast/ToastContext";

type ArchivedInventoryItemProps = {
  householdId: string;
  item: InventoryItem;
  onChanged: () => Promise<void>;
};

export function ArchivedInventoryItemRow({
  householdId,
  item,
  onChanged,
}: ArchivedInventoryItemProps) {
  const { showToast } = useToast();

  const [isRestoring, setIsRestoring] = useState(false);
  async function handleRestore() {
    setIsRestoring(false);

    await restoreInventoryItem(householdId, item.id)
      .then(() => onChanged())
      .finally(() => {
        setIsRestoring(false);
        showToast("Item restoration successful", "success");
      });
  }

  return (
    <div className="archived-inventory-item-row">
      <div className="archived-inventory-item-row__info">
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
