import { apiJson, apiRequest } from "../../api/client";
import type { QrAction } from "./types";

export async function getQrActions(householdId: string): Promise<QrAction[]> {
  return apiJson<QrAction[]>(`/households/${householdId}/qr`, {
    method: "GET",
  });
}

export async function createQrAction(householdId: string): Promise<void> {
  await apiRequest(`/households/${householdId}/qr`, {
    method: "POST",
  });
}

export async function executeQrAction(
  actionId: string,
  deviceToken: string,
): Promise<void> {
  await apiRequest(`/device/qr/${actionId}/execute`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${deviceToken}`,
    },
    handleUnauthorized: false,
  });
}
