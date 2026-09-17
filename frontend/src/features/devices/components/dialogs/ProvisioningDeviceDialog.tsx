import { useMemo } from "react";
import { QRCodeSVG } from "qrcode.react";

import { Dialog } from "../../../../components/dialog/Dialog";

import styles from "./ProvisioningDeviceDialog.module.css";

type Props = {
  deviceName: string;
  deviceId: string;
  token: string;
  onClose: () => void;
};

export function ProvisioningDeviceDialog({
  deviceName,
  deviceId,
  token,
  onClose,
}: Props) {
  const qrValue = useMemo(
    () =>
      JSON.stringify({
        type: "aims-device-setup",
        version: 1,
        deviceId,
        token,
      }),
    [deviceId, token],
  );

  return (
    <Dialog
      title="Set up device"
      description="Scan this QR code with the device you want to connect."
      onClose={onClose}
    >
      <div className={styles.device}>
        <span className={styles.deviceLabel}>Device</span>
        <strong>{deviceName}</strong>
      </div>

      <div className={styles.qr}>
        <QRCodeSVG value={qrValue} size={220} level="M" />
      </div>

      <p className={`dialog__hint ${styles.hint}`}>
        Keep this screen open until the device has completed setup
      </p>

      <div className="dialog__actions">
        <button
          type="button"
          className="button button--primary"
          onClick={onClose}
        >
          Done
        </button>
      </div>
    </Dialog>
  );
}
