import type { InventoryItemPriority } from "./types";

export type InventoryPriorityOption = {
  value: InventoryItemPriority;
  label: string;
};

export const INVENTORY_PRIORITIES: InventoryPriorityOption[] = [
  { value: "default", label: "Default" },
  { value: "low", label: "Low" },
  { value: "medium", label: "Medium" },
  { value: "high", label: "High" },
];
