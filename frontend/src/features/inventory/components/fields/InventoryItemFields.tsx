import {
  Select,
  type SelectOption,
} from "../../../../components/select/Select";
import { PRIORITIES, type Priority } from "../../../../domain/priority";
import { CategorySelect } from "./CategorySelect";

type InventoryItemFieldsProps = {
  householdId: string;
  name: string;
  categoryId: string | null;
  reorderThreshold: number | "";
  priority: Priority;

  onNameChange: (name: string) => void;
  onCategoryChange: (categoryId: string | null) => void;
  onReorderThresholdChange: (reorderThreshold: number | "") => void;
  onPriorityChange: (priority: Priority) => void;

  nameError?: string;
  reorderThresholdError?: string;

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
  nameError,
  reorderThresholdError,
  disabled = false,
  children,
}: InventoryItemFieldsProps) {
  return (
    <div className="dialog__fields">
      <label
        className={`dialog__field ${nameError ? "dialog__field--error" : ""}`}
      >
        <span>Name</span>

        <input
          type="text"
          value={name}
          onChange={(event) => onNameChange(event.target.value)}
          disabled={disabled}
        />

        {nameError && (
          <span className="dialog__field-error" role="alert">
            {nameError}
          </span>
        )}
      </label>

      <div className="dialog__field">
        <span>Category</span>

        <CategorySelect
          householdId={householdId}
          value={categoryId}
          onValueChange={onCategoryChange}
          disabled={disabled}
        />
      </div>

      <label
        className={`dialog__field ${
          reorderThresholdError ? "dialog__field--error" : ""
        }`}
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
          <span className="dialog__field-error" role="alert">
            {reorderThresholdError}
          </span>
        )}
      </label>

      <div className="dialog__field">
        <span>Priority</span>

        <Select
          value={priority}
          options={PRIORITIES as SelectOption<Priority>[]}
          onValueChange={(value) => onPriorityChange(value as Priority)}
          portal={false}
          disabled={disabled}
          ariaLabel="Priority"
        />
      </div>

      {children}
    </div>
  );
}
