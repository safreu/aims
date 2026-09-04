import type { InventoryItemPriority } from "../../../inventory/types";

type ShoppingEntryFieldsProps = {
  title: string;
  quantity: number;
  priority: InventoryItemPriority;
  note: string;

  onTitleChange: (name: string) => void;
  onQuantityChange: (reorderThreshold: number) => void;
  onPriorityChange: (priority: InventoryItemPriority) => void;
  onNoteChange: (note: string) => void;

  disabled?: boolean;
};

export function ShoppingEntryFields({
  title,
  quantity,
  priority,
  note,
  onTitleChange,
  onQuantityChange,
  onPriorityChange,
  onNoteChange,
  disabled = false,
}: ShoppingEntryFieldsProps) {
  return (
    <div className="inventory-item-dialog__fields">
      <label className="inventory-item-dialog__field">
        <span>Name</span>

        <input
          type="text"
          value={title}
          onChange={(event) => onTitleChange(event.target.value)}
          disabled={disabled}
        />
      </label>

      <label className="inventory-item-dialog__field">
        <span>Quantity</span>

        <input
          type="number"
          min="1"
          value={quantity}
          onChange={(event) => onQuantityChange(Number(event.target.value))}
          disabled={disabled}
        />
      </label>

      <label className="inventory-item-dialog__field">
        <span>Priority</span>

        <select
          value={priority}
          onChange={(event) =>
            onPriorityChange(event.target.value as InventoryItemPriority)
          }
          disabled={disabled}
        >
          <option value="default">Default</option>
          <option value="low">Low</option>
          <option value="medium">Medium</option>
          <option value="high">High</option>
        </select>
      </label>

      <label className="inventory-item-dialog__field">
        <span>Note</span>

        <input
          type="text"
          value={note}
          onChange={(event) => onNoteChange(event.target.value)}
          disabled={disabled}
        />
      </label>
    </div>
  );
}
