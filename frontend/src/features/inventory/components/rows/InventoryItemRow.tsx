import { useState } from "react";

import type { InventoryItem } from "../../types";
import { InventoryItemDialog } from "../dialogs/InventoryItemDialog";

import styles from "./InventoryItemRow.module.css";

type Props = {
  householdId: string;
  item: InventoryItem;
  onChanged: () => Promise<void>;
};

export function InventoryItemRow({ householdId, item, onChanged }: Props) {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <>
      <button
        type="button"
        className={styles.row}
        onClick={() => setIsOpen(true)}
      >
        <div className={styles.info}>
          <strong>{item.name}</strong>

          <span>{item.category?.name ?? "No category"}</span>
        </div>

        <div className={styles.stock}>
          <span>Stock</span>
          <strong>{item.current_stock}</strong>
        </div>
      </button>

      {isOpen && (
        <InventoryItemDialog
          householdId={householdId}
          itemId={item.id}
          onChanged={onChanged}
          onClose={() => setIsOpen(false)}
        />
      )}
    </>
  );
}
