export type InventoryItemPriority = "default" | "low" | "medium" | "high";
export type InventoryItemStatus = "active" | "archived";

export type InventoryItemCategory = {
  id: string;
  name: string;
};
export type InventoryItem = {
  id: string;
  name: string;
  category: InventoryItemCategory | null;
  current_stock: number;
  reorder_threshold: number;
  priority: InventoryItemPriority;
  shopping_quantity: number;
};

export type CreateInventoryItemRequest = {
  category_id: string | null;
  name: string;
  current_stock: number;
  reorder_threshold: number;
  priority: InventoryItemPriority;
};

export type CreateInventoryItemResponse = {
  id: string;
};

export type CreateInventoryCategoryRequest = {
  name: string;
};

export type CreateInventoryCategoryResponse = {
  id: string;
};

export type UpdateInventoryItemRequest = {
  name?: string;
  category_id?: string | null;
  reorder_threshold?: number | null;
  priority?: InventoryItemPriority;
};

export type ChangeInventoryStockRequest = {
  amount: number;
};

export type SetInventoryStockRequest = {
  stock: number;
};

export type InventoryStockHistoryEntry = {
  id: string;
  sequence_number: number;
  item_id: string;
  kind: string;
  source: string;
  amount: number | null;
  stock_before: number;
  stock_after: number;
  actor: InventoryStockHistoryActor;
  created_at: string;
};

export type InventoryStockHistoryActor =
  | { type: "user"; id: string; display_name: string }
  | { type: "device"; id: string; name: string }
  | { type: "system" };
