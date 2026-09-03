import * as RadixDropdownMenu from "@radix-ui/react-dropdown-menu";
import type { ReactNode } from "react";

import "./DropdownMenu.css";

type DropdownMenuProps = {
  trigger: ReactNode;
  children: ReactNode;
  portal?: boolean;
};

type DropdownMenuItemProps = {
  children: ReactNode;
  onSelect?: (event: Event) => void;
  disabled?: boolean;
  className?: string;
};

export function DropdownMenu({
  trigger,
  children,
  portal = true,
}: DropdownMenuProps) {
  const content = (
    <RadixDropdownMenu.Content
      className="dropdown-menu__content"
      sideOffset={6}
      align="start"
    >
      {children}
    </RadixDropdownMenu.Content>
  );
  return (
    <RadixDropdownMenu.Root>
      <RadixDropdownMenu.Trigger asChild>{trigger}</RadixDropdownMenu.Trigger>

      {portal ? (
        <RadixDropdownMenu.Portal>{content}</RadixDropdownMenu.Portal>
      ) : (
        content
      )}
    </RadixDropdownMenu.Root>
  );
}

export function DropdownMenuItem({
  children,
  onSelect,
  disabled = false,
  className = "",
}: DropdownMenuItemProps) {
  return (
    <RadixDropdownMenu.Item
      className={`dropdown-menu__item ${className}`}
      onSelect={onSelect}
      disabled={disabled}
    >
      {children}
    </RadixDropdownMenu.Item>
  );
}

export function DropDownMenuSeparator() {
  return <RadixDropdownMenu.Separator className="dropdown-menu__separator" />;
}
