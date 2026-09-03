import type { InventoryItemPriority } from "../../types";
import { CategorySelect } from "./CategorySelect";

type InventoryItemFieldsProps = {
  householdId: string;
  name: string;
  categoryId: string | null;
  reorderThreshold: number | "";
  priority: InventoryItemPriority;
  onNameChange: (name: string) => void;
  onCategoryChange: (categoryId: string | null) => void;
  onReorderThresholdChange: (reorderThreshold: number | "") => void;
  onPriorityChange: (priority: InventoryItemPriority) => void;

  disabled?: boolean;

  children?: React.ReactNode;
};

export function InventoryItemFields({
  householdId,
  name,
  categoryId,
  reorderThreshold,
  priority,
  onNameChange,
  onCategoryChange,
  onReorderThresholdChange,
  onPriorityChange,
  disabled = false,
  children,
}: InventoryItemFieldsProps) {
  return (
    <div className="inventory-item-dialog__fields">
      <label className="inventory-item-dialog__field">
        <span>Name</span>

        <input
          type="text"
          value={name}
          onChange={(event) => onNameChange(event.target.value)}
          disabled={disabled}
          required
        />
      </label>

      <div className="inventory-item-dialog__field">
        <span>Category</span>

        <CategorySelect
          householdId={householdId}
          value={categoryId}
          onValueChange={onCategoryChange}
        />
      </div>

      <label className="inventory-item-dialog__field">
        <span>Reorder threshold</span>

        <input
          type="number"
          min="0"
          value={reorderThreshold}
          onChange={(event) => {
            const value = event.target.value;

            onReorderThresholdChange(value === "" ? "" : Number(value));
          }}
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

        {children}
      </label>
    </div>
  );
}
