import { Check, ChevronDown } from "lucide-react";
import type { ReactNode } from "react";

import { DropdownMenu, DropdownMenuItem } from "../dropdown-menu/DropdownMenu";
import styles from "./Select.module.css";

export type SelectOption<T extends string> = {
  value: T;
  label: string;
  content?: ReactNode;
};

type SelectProps<T extends string> = {
  value: T;
  options: SelectOption<T>[];
  onValueChange: (value: T) => void;
  portal?: boolean;
  placeholder?: string;
  disabled?: boolean;
  ariaLabel?: string;
};

export function Select<T extends string>({
  value,
  options,
  onValueChange,
  portal = true,
  placeholder,
  disabled = false,
  ariaLabel,
}: SelectProps<T>) {
  const selectedOption = options.find((option) => option.value === value);

  return (
    <DropdownMenu
      portal={portal}
      trigger={
        <button
          type="button"
          className={styles.trigger}
          disabled={disabled}
          aria-label={ariaLabel}
        >
          <span className={styles.value}>
            {selectedOption?.label ?? placeholder}
          </span>

          <ChevronDown className={styles.icon} />
        </button>
      }
    >
      {options.map((option) => (
        <DropdownMenuItem
          key={option.value}
          onSelect={() => onValueChange(option.value)}
        >
          <span className={styles.optionLabel}>
            {option.content ?? option.label}
          </span>

          {option.value === value && <Check className={styles.indicator} />}
        </DropdownMenuItem>
      ))}
    </DropdownMenu>
  );
}
