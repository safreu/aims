export type DeviceKind = "scanner" | "display" | "smartphone" | "other";

export type Device = {
  id: string;
  name: string;
  kind: DeviceKind;
};

export type RegisterDeviceRequest = {
  name: string;
  kind: DeviceKind;
};

export type RegisterDeviceResponse = {
  id: string;
};

export type DeviceCredentialResponse = {
  token: string;
};

export type RenameDeviceRequest = {
  name: string;
};
