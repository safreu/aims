import { useState } from "react";
import { type InventoryItem } from "../../types";
import { InventoryItemDialog } from "../dialogs/InventoryItemDialog";
import "./InventoryItemRow.css";

type InventoryItemRowProps = {
  householdId: string;
  item: InventoryItem;
  onChanged: () => Promise<void>;
};

export function InventoryItemRow({
  householdId,
  item,
  onChanged,
}: InventoryItemRowProps) {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <>
      <button
        type="button"
        className="inventory-item-row"
        onClick={() => setIsOpen(true)}
      >
        <div className="inventory-item-row__info">
          <strong>{item.name}</strong>

          <span>{item.category?.name ?? "No category"}</span>
        </div>

        <div className="inventory-item-row__stock">
          <span>stock</span>
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
