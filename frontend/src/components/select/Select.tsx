import { Check, ChevronDown } from "lucide-react";

import "./Select.css";
import { DropdownMenu, DropdownMenuItem } from "../dropdown-menu/DropdownMenu";
import type { ReactNode } from "react";

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
          className="select__trigger"
          disabled={disabled}
          aria-label={ariaLabel}
        >
          <span className="select__value">
            {selectedOption?.label ?? placeholder}
          </span>

          <ChevronDown className="select__icon" />
        </button>
      }
    >
      {options.map((option) => (
        <DropdownMenuItem
          key={option.value}
          onSelect={() => onValueChange(option.value)}
        >
          <span className="select__option-label">
            {option.content ?? option.label}
          </span>

          {option.value === value && <Check className="select__indicator" />}
        </DropdownMenuItem>
      ))}
    </DropdownMenu>
  );
}
