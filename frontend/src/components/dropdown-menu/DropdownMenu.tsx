import * as RadixDropdownMenu from "@radix-ui/react-dropdown-menu";
import type { ReactNode } from "react";

import styles from "./DropdownMenu.module.css";

type DropdownMenuProps = {
  trigger: ReactNode;
  children: ReactNode;
  portal?: boolean;
  onOpenChange?: (open: boolean) => void;
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
  onOpenChange,
}: DropdownMenuProps) {
  const content = (
    <RadixDropdownMenu.Content
      className={styles.content}
      sideOffset={6}
      align="start"
    >
      {children}
    </RadixDropdownMenu.Content>
  );

  return (
    <RadixDropdownMenu.Root onOpenChange={onOpenChange}>
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
      className={`${styles.item} ${className}`}
      onSelect={onSelect}
      disabled={disabled}
    >
      {children}
    </RadixDropdownMenu.Item>
  );
}

export function DropdownMenuSeparator() {
  return <RadixDropdownMenu.Separator className={styles.separator} />;
}

type DropdownMenuCheckboxItemProps = {
  children: ReactNode;
  checked: boolean;
  onCheckedChange: (checked: boolean) => void;
};

export function DropdownMenuCheckboxItem({
  children,
  checked,
  onCheckedChange,
}: DropdownMenuCheckboxItemProps) {
  return (
    <RadixDropdownMenu.CheckboxItem
      className={styles.item}
      checked={checked}
      onCheckedChange={(checked) => {
        if (typeof checked === "boolean") onCheckedChange(checked);
      }}
    >
      {children}
    </RadixDropdownMenu.CheckboxItem>
  );
}

type DropdownMenuItemContentProps = {
  children: ReactNode;
};

export function DropdownMenuItemContent({
  children,
}: DropdownMenuItemContentProps) {
  return <div className={styles.itemContent}>{children}</div>;
}
