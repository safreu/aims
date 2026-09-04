import { Check } from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuItem,
} from "../../../../components/dropdown-menu/DropdownMenu";
import { INVENTORY_PRIORITIES } from "../../priorities";
import type { InventoryItemPriority } from "../../types";
import { PriorityIndicator } from "./PriorityIndicator";
import "./PrioritySelect.css";

type Props = {
  value: InventoryItemPriority;
  onValueChange: (value: InventoryItemPriority) => void;
  disabled?: boolean;
};

export function PrioritySelect({ value, onValueChange, disabled }: Props) {
  return (
    <DropdownMenu
      trigger={
        <button
          type="button"
          className="priority-select__trigger"
          disabled={disabled}
          aria-label="Change priority"
        >
          <PriorityIndicator priority={value} />
        </button>
      }
    >
      {INVENTORY_PRIORITIES.map((priority) => (
        <DropdownMenuItem
          key={priority.value}
          onSelect={() => onValueChange(priority.value)}
        >
          <PriorityIndicator priority={priority.value} />
          {priority.value === value && (
            <Check className="priority-select__check" />
          )}
        </DropdownMenuItem>
      ))}
    </DropdownMenu>
  );
}
