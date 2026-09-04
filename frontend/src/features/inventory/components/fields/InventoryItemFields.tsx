import {
  Select,
  type SelectOption,
} from "../../../../components/select/Select";
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

  nameError?: string;
  reorderThresholdError?: string;

  disabled?: boolean;

  children?: React.ReactNode;
};

const PriorityOptions: SelectOption<InventoryItemPriority>[] = [
  { value: "default", label: "Default" },
  { value: "low", label: "Low" },
  { value: "medium", label: "Medium" },
  { value: "high", label: "High" },
];

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
  nameError,
  reorderThresholdError,
  disabled = false,
  children,
}: InventoryItemFieldsProps) {
  return (
    <div className="inventory-item-dialog__fields">
      <label
        className={`inventory-item-dialog__field ${nameError ? "inventory-item-dialog__field--error" : ""}`}
      >
        <span>Name</span>

        <input
          type="text"
          value={name}
          onChange={(event) => onNameChange(event.target.value)}
          disabled={disabled}
        />
        {nameError && (
          <span className="inventory-item-dialog__field-error">
            {nameError}
          </span>
        )}
      </label>

      <div className="inventory-item-dialog__field">
        <span>Category</span>

        <CategorySelect
          householdId={householdId}
          value={categoryId}
          onValueChange={onCategoryChange}
        />
      </div>

      <label
        className={`inventory-item-dialog__field ${reorderThresholdError ? "inventory-item-dialog__field--error" : ""}`}
      >
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
        {reorderThresholdError && (
          <span className="inventory-item-dialog__field-error">
            {reorderThresholdError}
          </span>
        )}
      </label>

      <div className="inventory-item-dialog__field">
        <span>Priority</span>

        <Select
          value={priority}
          options={PriorityOptions}
          onValueChange={onPriorityChange}
          portal={false}
          disabled={disabled}
          ariaLabel="Priority"
        />
      </div>
      {children}
    </div>
  );
}
