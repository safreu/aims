import { useCallback, useEffect, useState } from "react";
import type { Device, DeviceKind } from "../types";
import {
  getLocalDeviceCredentials,
  removeLocalDeviceCredential,
  saveLocalDeviceCredential,
} from "../localDevice";
import {
  getDevices,
  issueDeviceCredential,
  registerDevice,
  renameDevice,
  revokeDevice,
  rotateDeviceCredential,
} from "../api";
import { useToast } from "../../../components/toast/ToastContext";
import { RegisterCurrentDeviceDialog } from "./Dialogs/RegisterCurrentDeviceDialog";

import "./DeviceList.css";
import { ConfirmDialog } from "../../../components/dialogs/ConfirmDialog";
import { RenameDeviceDialog } from "./Dialogs/RenameDeviceDialog";
import { RegisterOtherDeviceDialog } from "./Dialogs/RegisterOtherDeviceDialog";
import { ProvisioningDeviceDialog } from "./Dialogs/ProvisioningDeviceDialog";

type Props = {
  householdId: string;
};

type ProvisingDevice = {
  deviceId: string;
  deviceName: string;
  token: string;
};

export function DeviceList({ householdId }: Props) {
  const { showToast } = useToast();

  const [showRegisterDeviceDialog, setShowRegisterDeviceDialog] =
    useState(false);
  const [showRegisterOtherDeviceDialog, setShowRegisterOtherDeviceDialog] =
    useState(false);

  const [devices, setDevices] = useState<Device[]>([]);

  const [deviceToRevoke, setDeviceToRevoke] = useState<Device | null>(null);
  const [isRevoking, setIsRevoking] = useState(false);

  const [deviceToRename, setDeviceToRename] = useState<Device | null>(null);
  const [isRenaming, setIsRenaming] = useState(false);

  const [provisioningDevice, setProvisioningDevice] =
    useState<ProvisingDevice | null>(null);

  const [openActionsDeviceId, setOpenActionsDeviceId] = useState<string | null>(
    null,
  );

  const [loading, setLoading] = useState(true);
  const [isRegistering, setIsRegistering] = useState(false);
  const [isRegisteringOtherDevice, setIsRegisteringOtherDevice] =
    useState(false);

  const [isRotatingCredential, setIsRotatingCredential] = useState(false);

  const localCredential = getLocalDeviceCredentials(householdId);

  const currentDevice = devices.find(
    (device) => device.id === localCredential?.deviceId,
  );

  const refreshDevice = useCallback(async () => {
    setLoading(true);

    await getDevices(householdId)
      .then((devices) => setDevices(devices))
      .catch(() => showToast("Failed to fetch devices", "error"))
      .finally(() => setLoading(false));
  }, [householdId, showToast]);

  async function handleRegisterCurrentDevice(name: string) {
    let deviceId: string;

    setIsRegistering(true);

    return registerDevice(householdId, { name, kind: "smartphone" })
      .then((registered) => {
        deviceId = registered.id;

        return issueDeviceCredential(householdId, registered.id);
      })
      .then((credential) => {
        saveLocalDeviceCredential(householdId, {
          deviceId,
          token: credential.token,
        });

        return refreshDevice();
      })
      .then(() => {
        setShowRegisterDeviceDialog(false);
        showToast("Device registered", "success");
      })
      .catch(() => showToast("Failed to register device", "error"))
      .finally(() => setIsRegistering(false));
  }

  async function handleRevokeDevice() {
    if (deviceToRevoke === null) return;

    setIsRevoking(true);

    return revokeDevice(householdId, deviceToRevoke.id)
      .then(() => {
        if (deviceToRevoke.id === localCredential?.deviceId) {
          removeLocalDeviceCredential(householdId);
        }
        return refreshDevice();
      })
      .then(() => {
        showToast("Device revoked", "success");
        setDeviceToRevoke(null);
      })
      .catch(() => showToast("Failed to revoke device", "error"))
      .finally(() => setIsRevoking(false));
  }

  async function handleRenameDevice(name: string) {
    if (deviceToRename === null) return Promise.resolve();

    setIsRenaming(true);

    return renameDevice(householdId, deviceToRename.id, {
      name,
    })
      .then(() => refreshDevice())
      .then(() => {
        showToast("Device renamed", "success");
        setDeviceToRename(null);
      })
      .finally(() => setIsRenaming(false));
  }

  async function handleRegisterOtherDevice(name: string, kind: DeviceKind) {
    setIsRegisteringOtherDevice(true);

    return registerDevice(householdId, { name, kind })
      .then((registered) =>
        issueDeviceCredential(householdId, registered.id).then(
          (credential) => ({
            deviceId: registered.id,
            deviceName: name,
            token: credential.token,
          }),
        ),
      )
      .then((provisioningData) => {
        setProvisioningDevice(provisioningData);
        return refreshDevice();
      })
      .then(() => {
        setShowRegisterOtherDeviceDialog(false);
        showToast("Device registered", "success");
      })
      .catch(() => showToast("Failed to register device", "error"))
      .finally(() => setIsRegisteringOtherDevice(false));
  }

  async function handleShowSetupQr(device: Device) {
    setIsRotatingCredential(true);

    return rotateDeviceCredential(householdId, device.id)
      .then((credential) => {
        setProvisioningDevice({
          deviceId: device.id,
          deviceName: device.name,
          token: credential.token,
        });
        setOpenActionsDeviceId(null);
      })
      .catch(() => showToast("Failed to generate setup QR code", "error"))
      .finally(() => setIsRotatingCredential(false));
  }

  useEffect(() => {
    void getDevices(householdId)
      .then((devices) => setDevices(devices))
      .catch(() => showToast("Failed to fetch devices", "error"))
      .finally(() => setLoading(false));
  }, [householdId, showToast]);

  if (loading) {
    return <p>Loading devices...</p>;
  }

  return (
    <div className="device-list">
      <button
        type="button"
        className="button button--secondary"
        onClick={() => setShowRegisterOtherDeviceDialog(true)}
      >
        Register another device
      </button>

      {devices.length === 0 ? (
        <p className="device-list__empty">No devices registered</p>
      ) : (
        devices.map((device) => {
          const isCurrentDevice = device.id === currentDevice?.id;

          return (
            <div key={device.id} className="device-list__item">
              <div className="device-list__item-info">
                <span className="device-list__item-name">{device.name}</span>

                <div className="device-list__item-meta">
                  <span className="device-list__item-kind">{device.kind}</span>

                  {isCurrentDevice && (
                    <span className="device-list__badge">This device</span>
                  )}
                </div>
              </div>

              <div className="device-list__actions-wrapper">
                <button
                  type="button"
                  className="device-list__manage-button"
                  onClick={() =>
                    setOpenActionsDeviceId((current) =>
                      current === device.id ? null : device.id,
                    )
                  }
                  aria-label={`Manage ${device.id}`}
                  aria-expanded={openActionsDeviceId === device.id}
                >
                  ...
                </button>

                {openActionsDeviceId === device.id && (
                  <div className="device-list__actions-menu">
                    <button
                      type="button"
                      className="device-list__menu-action"
                      onClick={() => {
                        setOpenActionsDeviceId(null);
                        setDeviceToRename(device);
                      }}
                    >
                      Rename
                    </button>

                    {!isCurrentDevice && (
                      <button
                        type="button"
                        className="device-list__menu-action"
                        disabled={isRotatingCredential}
                        onClick={() => void handleShowSetupQr(device)}
                      >
                        Show setup QR
                      </button>
                    )}

                    <button
                      type="button"
                      className="device-list__menu-action device-list__menu-action--danger"
                      onClick={() => {
                        setOpenActionsDeviceId(null);
                        setDeviceToRevoke(device);
                      }}
                    >
                      Revoke
                    </button>
                  </div>
                )}
              </div>
            </div>
          );
        })
      )}

      {!currentDevice && (
        <div className="device-list__register">
          <div className="device-list__register-info">
            <strong>This device</strong>
            <p>Register this phone to use QR scanning.</p>
          </div>

          <button
            type="button"
            className="button button--primary"
            onClick={() => setShowRegisterDeviceDialog(true)}
          >
            Register
          </button>
        </div>
      )}

      {showRegisterDeviceDialog && (
        <RegisterCurrentDeviceDialog
          registering={isRegistering}
          onRegister={handleRegisterCurrentDevice}
          onClose={() => setShowRegisterDeviceDialog(false)}
        />
      )}

      <ConfirmDialog
        open={deviceToRevoke !== null}
        title="Revoke device?"
        description={
          deviceToRevoke
            ? `Revoke "${deviceToRevoke.name}"? This device will no longer be able to authenticate`
            : ""
        }
        confirmLabel="Revoke device"
        destructive
        loading={isRevoking}
        onConfirm={handleRevokeDevice}
        onCancel={() => setDeviceToRevoke(null)}
      />

      {deviceToRename && (
        <RenameDeviceDialog
          device={deviceToRename}
          renaming={isRenaming}
          onRename={handleRenameDevice}
          onClose={() => setDeviceToRename(null)}
        />
      )}

      {showRegisterOtherDeviceDialog && (
        <RegisterOtherDeviceDialog
          registering={isRegisteringOtherDevice}
          onRegister={handleRegisterOtherDevice}
          onClose={() => setShowRegisterOtherDeviceDialog(false)}
        />
      )}

      {provisioningDevice && (
        <ProvisioningDeviceDialog
          deviceName={provisioningDevice.deviceName}
          deviceId={provisioningDevice.deviceId}
          token={provisioningDevice.token}
          onClose={() => setProvisioningDevice(null)}
        />
      )}
    </div>
  );
}
