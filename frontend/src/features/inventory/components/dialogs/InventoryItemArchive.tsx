import { useState } from "react";
import { useToast } from "../../../../components/toast/ToastContext";
import type { InventoryItem } from "../../types";
import { archiveInventoryItem } from "../../api";

type Props = {
  householdId: string;
  item: InventoryItem;
  onArchived: () => void;
};

export function InventoryItemArchive({ householdId, item, onArchived }: Props) {
  const { showToast } = useToast();

  const [isArchiving, setIsArchiving] = useState(false);

  function handleArchive() {
    setIsArchiving(true);

    void archiveInventoryItem(householdId, item.id)
      .then(() => {
        showToast("Item archived", "success");
        onArchived();
      })
      .catch(() => showToast("Failed to archive item", "error"))
      .finally(() => setIsArchiving(false));
  }

  return (
    <section className="inventory-item-dialog__section inventory-item-dialog__danger">
      <h3>Archive item</h3>
      <p>The item will disappear from the active inventory</p>

      <button
        type="button"
        className="button button--danger"
        onClick={handleArchive}
        disabled={isArchiving}
      >
        Archive item
      </button>
    </section>
  );
}
