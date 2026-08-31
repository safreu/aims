import { apiJson, apiRequest } from "../../api/client";
import type {
  ChangeInventoryStockRequest,
  CreateInventoryCategoryRequest,
  CreateInventoryCategoryResponse,
  CreateInventoryItemRequest,
  CreateInventoryItemResponse,
  InventoryItem,
  InventoryItemCategory,
  InventoryItemStatus,
  InventoryStockHistoryEntry,
  SetInventoryStockRequest,
  UpdateInventoryItemRequest,
} from "./types";

export async function getInventoryItems(
  householdId: string,
  status: InventoryItemStatus = "active",
): Promise<InventoryItem[]> {
  return apiJson<InventoryItem[]>(
    `/inventory/${householdId}/items?status=${status}`,
  );
}

export async function createInventoryItem(
  householdId: string,
  request: CreateInventoryItemRequest,
): Promise<CreateInventoryItemResponse> {
  return apiJson<CreateInventoryItemResponse>(
    `/inventory/${householdId}/items`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(request),
    },
  );
}

export async function getInventoryCategories(
  householdId: string,
): Promise<InventoryItemCategory[]> {
  return apiJson<InventoryItemCategory[]>(
    `/inventory/${householdId}/categories`,
  );
}

export async function createInventoryCategory(
  householdId: string,
  request: CreateInventoryCategoryRequest,
): Promise<CreateInventoryCategoryResponse> {
  return apiJson<CreateInventoryCategoryResponse>(
    `/inventory/${householdId}/categories`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(request),
    },
  );
}

export async function updateInventoryItem(
  householdId: string,
  itemId: string,
  request: UpdateInventoryItemRequest,
): Promise<void> {
  apiRequest(`/inventory/${householdId}/items/${itemId}`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(request),
  });
}

export async function increaseInventoryStock(
  householdId: string,
  itemId: string,
  request: ChangeInventoryStockRequest,
): Promise<void> {
  await apiRequest(`/inventory/${householdId}/items/${itemId}/increase`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(request),
  });
}

export async function decreaseInventoryStock(
  householdId: string,
  itemId: string,
  request: ChangeInventoryStockRequest,
): Promise<void> {
  await apiRequest(`/inventory/${householdId}/items/${itemId}/decrease`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(request),
  });
}

export async function setInventoryStock(
  householdId: string,
  itemId: string,
  request: SetInventoryStockRequest,
): Promise<void> {
  await apiRequest(`/inventory/${householdId}/items/${itemId}/stock`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(request),
  });
}

export async function archiveInventoryItem(
  householdId: string,
  itemId: string,
): Promise<void> {
  await apiRequest(`/inventory/${householdId}/items/${itemId}/archive`, {
    method: "POST",
  });
}

export async function restoreInventoryItem(
  householdId: string,
  itemId: string,
): Promise<void> {
  await apiRequest(`/inventory/${householdId}/items/${itemId}/restore`, {
    method: "POST",
  });
}

export async function getInventoryStockHistory(
  householdId: string,
  itemId: string,
): Promise<InventoryStockHistoryEntry[]> {
  return apiJson(`/inventory/${householdId}/items/${itemId}/history`);
}
