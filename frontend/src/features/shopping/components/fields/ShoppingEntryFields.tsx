import {
  Select,
  type SelectOption,
} from "../../../../components/select/Select";
import type { Priority } from "../../../../domain/priority";

const PRIORITY_OPTIONS: SelectOption<Priority>[] = [
  { value: "default", label: "Default" },
  { value: "low", label: "Low" },
  { value: "medium", label: "Medium" },
  { value: "high", label: "High" },
];

type ShoppingEntryFieldsProps = {
  title: string;
  quantity: number | "";
  priority: Priority;
  note: string;

  onTitleChange: (title: string) => void;
  onQuantityChange: (quantity: number | "") => void;
  onPriorityChange: (priority: Priority) => void;
  onNoteChange: (note: string) => void;

  titleError?: string;
  quantityError?: string;

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
  titleError,
  quantityError,
  disabled = false,
}: ShoppingEntryFieldsProps) {
  return (
    <div className="dialog__fields">
      <label
        className={`dialog__field ${titleError ? "dialog__field--error" : ""}`}
      >
        <span>Name</span>

        <input
          type="text"
          value={title}
          onChange={(event) => onTitleChange(event.target.value)}
          disabled={disabled}
        />

        {titleError && (
          <span className="dialog__field-error" role="alert">
            {titleError}
          </span>
        )}
      </label>

      <label
        className={`dialog__field ${
          quantityError ? "dialog__field--error" : ""
        }`}
      >
        <span>Quantity</span>

        <input
          type="number"
          min="1"
          step="1"
          value={quantity}
          onChange={(event) => {
            const value = event.target.value;

            onQuantityChange(value === "" ? "" : Number(value));
          }}
          disabled={disabled}
        />

        {quantityError && (
          <span className="dialog__field-error" role="alert">
            {quantityError}
          </span>
        )}
      </label>

      <label className="dialog__field">
        <span>Priority</span>

        <Select
          value={priority}
          options={PRIORITY_OPTIONS}
          onValueChange={(value) => onPriorityChange(value as Priority)}
          portal={false}
          disabled={disabled}
          ariaLabel="Priority"
        />
      </label>

      <label className="dialog__field">
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
