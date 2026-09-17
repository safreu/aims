import { Check } from "lucide-react";

import { DropdownMenu, DropdownMenuItem } from "../dropdown-menu/DropdownMenu";
import { PRIORITIES, type Priority } from "../../domain/priority";
import { PriorityIndicator } from "./PriorityIndicator";

import styles from "./PrioritySelect.module.css";

type Props = {
  value: Priority;
  onValueChange: (value: Priority) => void;
  disabled?: boolean;
};

export function PrioritySelect({ value, onValueChange, disabled }: Props) {
  return (
    <DropdownMenu
      trigger={
        <button
          type="button"
          className={styles.trigger}
          disabled={disabled}
          aria-label="Change priority"
        >
          <PriorityIndicator priority={value} />
        </button>
      }
    >
      {PRIORITIES.map((priority) => (
        <DropdownMenuItem
          key={priority.value}
          onSelect={() => onValueChange(priority.value as Priority)}
        >
          <PriorityIndicator priority={priority.value} showLabel />

          {priority.value === value && (
            <Check className={styles.check} aria-hidden="true" />
          )}
        </DropdownMenuItem>
      ))}
    </DropdownMenu>
  );
}
