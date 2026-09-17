import { useQuery, useQueryClient } from "@tanstack/react-query";
import { MoreVertical } from "lucide-react";
import { useState } from "react";
import Skeleton from "react-loading-skeleton";

import { queryKeys } from "../../../api/queryKeys";
import { ConfirmDialog } from "../../../components/dialog/ConfirmDialog";
import {
  DropdownMenu,
  DropdownMenuItem,
} from "../../../components/dropdown-menu/DropdownMenu";
import { useToast } from "../../../components/toast/ToastContext";
import {
  getDevices,
  issueDeviceCredential,
  registerDevice,
  renameDevice,
  revokeDevice,
  rotateDeviceCredential,
} from "../api";
import {
  getLocalDeviceCredentials,
  removeLocalDeviceCredential,
} from "../localDevice";
import type { Device, DeviceKind } from "../types";
import { ProvisioningDeviceDialog } from "./dialogs/ProvisioningDeviceDialog";
import { RegisterOtherDeviceDialog } from "./dialogs/RegisterOtherDeviceDialog";
import { RenameDeviceDialog } from "./dialogs/RenameDeviceDialog";
import styles from "./DeviceList.module.css";

type Props = {
  householdId: string;
};

type ProvisioningDevice = {
  deviceId: string;
  deviceName: string;
  token: string;
};

export function DeviceList({ householdId }: Props) {
  const { showToast } = useToast();
  const queryClient = useQueryClient();

  const [showRegisterOtherDeviceDialog, setShowRegisterOtherDeviceDialog] =
    useState(false);

  const [deviceToRevoke, setDeviceToRevoke] = useState<Device | null>(null);
  const [isRevoking, setIsRevoking] = useState(false);

  const [deviceToRename, setDeviceToRename] = useState<Device | null>(null);
  const [isRenaming, setIsRenaming] = useState(false);

  const [provisioningDevice, setProvisioningDevice] =
    useState<ProvisioningDevice | null>(null);

  const [isRegisteringOtherDevice, setIsRegisteringOtherDevice] =
    useState(false);

  const [isRotatingCredential, setIsRotatingCredential] = useState(false);

  const localCredential = getLocalDeviceCredentials(householdId);

  const {
    data: devices = [],
    isPending,
    isError,
  } = useQuery({
    queryKey: queryKeys.devices(householdId),
    queryFn: () => getDevices(householdId),
  });

  const currentDevice = devices.find(
    (device) => device.id === localCredential?.deviceId,
  );

  async function handleRevokeDevice() {
    if (deviceToRevoke === null) return;

    setIsRevoking(true);

    return revokeDevice(householdId, deviceToRevoke.id)
      .then(() => {
        if (deviceToRevoke.id === localCredential?.deviceId) {
          removeLocalDeviceCredential(householdId);
        }

        return queryClient.invalidateQueries({
          queryKey: queryKeys.devices(householdId),
        });
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
      .then(() => {
        return queryClient.invalidateQueries({
          queryKey: queryKeys.devices(householdId),
        });
      })
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

        return queryClient.invalidateQueries({
          queryKey: queryKeys.devices(householdId),
        });
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
      })
      .catch(() => showToast("Failed to generate setup QR code", "error"))
      .finally(() => setIsRotatingCredential(false));
  }

  return (
    <div className={styles.list}>
      <button
        type="button"
        className="button button--secondary"
        onClick={() => setShowRegisterOtherDeviceDialog(true)}
      >
        Register another device
      </button>

      {isPending ? (
        <DeviceRowsSkeleton />
      ) : isError ? (
        <p className={styles.empty}>Failed to load devices</p>
      ) : devices.length === 0 ? (
        <p className={styles.empty}>No devices registered</p>
      ) : (
        devices.map((device) => {
          const isCurrentDevice = device.id === currentDevice?.id;

          return (
            <div key={device.id} className={styles.item}>
              <div className={styles.itemInfo}>
                <span className={styles.itemName}>{device.name}</span>

                <div className={styles.itemMeta}>
                  <span className={styles.itemKind}>{device.kind}</span>

                  {isCurrentDevice && (
                    <span className={styles.badge}>This device</span>
                  )}
                </div>
              </div>

              <DropdownMenu
                trigger={
                  <button
                    type="button"
                    className={styles.manageButton}
                    aria-label={`Manage ${device.name}`}
                  >
                    <MoreVertical />
                  </button>
                }
              >
                <DropdownMenuItem onSelect={() => setDeviceToRename(device)}>
                  Rename
                </DropdownMenuItem>

                {!isCurrentDevice && (
                  <DropdownMenuItem
                    disabled={isRotatingCredential}
                    onSelect={() => void handleShowSetupQr(device)}
                  >
                    Show setup QR
                  </DropdownMenuItem>
                )}

                <DropdownMenuItem
                  className={styles.dangerAction}
                  onSelect={() => setDeviceToRevoke(device)}
                >
                  Revoke
                </DropdownMenuItem>
              </DropdownMenu>
            </div>
          );
        })
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

function DeviceRowsSkeleton() {
  return (
    <>
      {Array.from({ length: 2 }).map((_, index) => (
        <div key={index} className={styles.item}>
          <div className={styles.itemInfo}>
            <Skeleton width="8rem" height="0.95rem" />

            <div className={styles.itemMeta}>
              <Skeleton width="4rem" height="0.75rem" />
            </div>
          </div>

          <Skeleton width={34} height={34} borderRadius="var(--radius-sm)" />
        </div>
      ))}
    </>
  );
}
