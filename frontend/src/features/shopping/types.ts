export type ShoppingPriority = "default" | "low" | "medium" | "high";

export type ShoppingCategory = {
  id: string;
  name: string;
};

export type InventoryShoppingEntry = {
  item_id: string;
  name: string;
  category: ShoppingCategory | null;
  quantity: number;
  priority: ShoppingPriority;
  note: string | null;
  checked: boolean;
};

export type CustomShoppingEntry = {
  id: string;
  title: string;
  quantity: number;
  priority: ShoppingPriority;
  note: string | null;
  checked: boolean;
};

export type ShoppingList = {
  inventory_entries: InventoryShoppingEntry[];
  custom_entries: CustomShoppingEntry[];
};

export type SetShoppingQuantityRequest = {
  quantity: number;
};

export type SetShoppingNoteRequest = {
  note: string | null;
};

export type SetShoppingCheckedRequest = {
  checked: boolean;
};

export type CreateCustomShoppingRequest = {
  title: string;
  quantity: number;
  priority: ShoppingPriority;
  note: string | null;
};

export type UpdateCustomShoppingRequest = {
  title?: string;
  quantity?: number;
  priority?: ShoppingPriority;
  note?: string | null;
};

export type SetCustomShoppingCheckedRequest = {
  checked: boolean;
};

export type CreateCustomShoppingResponse = {
  id: string;
};
