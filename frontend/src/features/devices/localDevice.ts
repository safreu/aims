export type LocalDeviceCredential = {
  deviceId: string;
  token: string;
};

function getStorageKey(householdId: string): string {
  return `aims-device:${householdId}`;
}

export function getLocalDeviceCredentials(
  householdId: string,
): LocalDeviceCredential | null {
  const stored = localStorage.getItem(getStorageKey(householdId));

  if (stored === null) return null;

  try {
    const value = JSON.parse(stored) as LocalDeviceCredential;

    if (typeof value.deviceId !== "string" || typeof value.token !== "string")
      return null;

    return value;
  } catch {
    return null;
  }
}

export function saveLocalDeviceCredential(
  householdId: string,
  credential: LocalDeviceCredential,
): void {
  localStorage.setItem(getStorageKey(householdId), JSON.stringify(credential));
}

export function removeLocalDeviceCredential(householdId: string): void {
  localStorage.removeItem(getStorageKey(householdId));
}
