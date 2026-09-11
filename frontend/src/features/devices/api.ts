import { apiJson, apiRequest } from "../../api/client";
import type {
  Device,
  DeviceCredentialResponse,
  RegisterDeviceRequest,
  RegisterDeviceResponse,
  RenameDeviceRequest,
} from "./types";

export async function getDevices(householdId: string): Promise<Device[]> {
  return apiJson<Device[]>(`/households/${householdId}/devices`, {
    method: "GET",
  });
}

export async function registerDevice(
  householdId: string,
  request: RegisterDeviceRequest,
): Promise<RegisterDeviceResponse> {
  return apiJson<RegisterDeviceResponse>(`/households/${householdId}/devices`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(request),
  });
}

export async function issueDeviceCredential(
  householdId: string,
  deviceId: string,
): Promise<DeviceCredentialResponse> {
  return apiJson<DeviceCredentialResponse>(
    `/households/${householdId}/devices/${deviceId}/credentials`,
    {
      method: "POST",
    },
  );
}

export async function revokeDevice(householdId: string, deviceId: string) {
  return apiRequest(`/households/${householdId}/devices/${deviceId}/revoke`, {
    method: "POST",
  });
}

export async function renameDevice(
  householdId: string,
  deviceId: string,
  request: RenameDeviceRequest,
) {
  return apiRequest(`/households/${householdId}/devices/${deviceId}`, {
    method: "PATCH",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(request),
  });
}

export async function rotateDeviceCredential(
  householdId: string,
  deviceId: string,
): Promise<DeviceCredentialResponse> {
  return apiJson<DeviceCredentialResponse>(
    `/households/${householdId}/devices/${deviceId}/credentials/rotate`,
    {
      method: "POST",
    },
  );
}
