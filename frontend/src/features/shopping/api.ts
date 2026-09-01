import { apiJson, apiRequest } from "../../api/client";
import type {
  CreateCustomShoppingRequest,
  CreateCustomShoppingResponse,
  SetCustomShoppingCheckedRequest,
  SetShoppingCheckedRequest,
  SetShoppingNoteRequest,
  SetShoppingQuantityRequest,
  ShoppingList,
  UpdateCustomShoppingRequest,
} from "./types";

export async function getShoppingList(
  householdId: string,
): Promise<ShoppingList> {
  return apiJson<ShoppingList>(`/households/${householdId}/shopping`);
}

export async function setShoppingQuantity(
  householdId: string,
  itemId: string,
  request: SetShoppingQuantityRequest,
): Promise<void> {
  await apiRequest(
    `/households/${householdId}/shopping/items/${itemId}/quantity`,
    {
      method: "PATCH",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(request),
    },
  );
}

export async function setShoppingNote(
  householdId: string,
  itemId: string,
  request: SetShoppingNoteRequest,
): Promise<void> {
  await apiRequest(`/households/${householdId}/shopping/items/${itemId}/note`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(request),
  });
}

export async function setShoppingChecked(
  householdId: string,
  itemId: string,
  request: SetShoppingCheckedRequest,
): Promise<void> {
  await apiRequest(
    `/households/${householdId}/shopping/items/${itemId}/checked`,
    {
      method: "PATCH",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(request),
    },
  );
}

export async function dismissShoppingItem(
  householdId: string,
  itemId: string,
): Promise<void> {
  await apiRequest(`/households/${householdId}/shopping/items/${itemId}`, {
    method: "DELETE",
  });
}

export async function createCustomShoppingEntry(
  householdId: string,
  request: CreateCustomShoppingRequest,
): Promise<CreateCustomShoppingResponse> {
  return apiJson<CreateCustomShoppingResponse>(
    `/households/${householdId}/shopping/custom`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(request),
    },
  );
}

export async function updateCustomShoppingEntry(
  householdId: string,
  entryId: string,
  request: UpdateCustomShoppingRequest,
): Promise<void> {
  await apiRequest(`/households/${householdId}/shopping/custom/${entryId}`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(request),
  });
}

export async function setCustomShoppingChecked(
  householdId: string,
  entryId: string,
  request: SetCustomShoppingCheckedRequest,
): Promise<void> {
  await apiRequest(
    `/households/${householdId}/shopping/custom/${entryId}/checked`,
    {
      method: "PATCH",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(request),
    },
  );
}

export async function deleteCustomShoppingEntry(
  householdId: string,
  entryId: string,
): Promise<void> {
  await apiRequest(`/households/${householdId}/shopping/custom/${entryId}`, {
    method: "DELETE",
  });
}
