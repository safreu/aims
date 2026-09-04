import * as RadixSelect from "@radix-ui/react-select";
import { Check, ChevronDown } from "lucide-react";

import "./Select.css";

type SelectOption<T extends string> = {
  value: T;
  label: string;
};

type SelectProps<T extends string> = {
  value: T;
  options: SelectOption<T>[];
  onValueChange: (value: T) => void;

  placeholder?: string;
  disabled?: boolean;
  ariaLabel?: string;
};

export function Select<T extends string>({
  value,
  options,
  onValueChange,
  placeholder,
  disabled = false,
  ariaLabel,
}: SelectProps<T>) {
  return (
    <RadixSelect.Root
      value={value}
      onValueChange={(value) => onValueChange(value as T)}
      disabled={disabled}
    >
      <RadixSelect.Trigger className="select__trigger" aria-label={ariaLabel}>
        <RadixSelect.Value placeholder={placeholder} />

        <RadixSelect.Icon className="select__icon">
          <ChevronDown />
        </RadixSelect.Icon>
      </RadixSelect.Trigger>

      <RadixSelect.Portal>
        <RadixSelect.Content
          className="select__content"
          position="popper"
          sideOffset={4}
        >
          <RadixSelect.Viewport className="select__viewport">
            {options.map((option) => (
              <RadixSelect.Item
                key={option.value}
                value={option.value}
                className="select__item"
              >
                <span className="select__item-indicator">
                  <RadixSelect.ItemIndicator>
                    <Check />
                  </RadixSelect.ItemIndicator>
                </span>

                <RadixSelect.ItemText>{option.label}</RadixSelect.ItemText>
              </RadixSelect.Item>
            ))}
          </RadixSelect.Viewport>
        </RadixSelect.Content>
      </RadixSelect.Portal>
    </RadixSelect.Root>
  );
}
