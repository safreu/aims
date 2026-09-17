import { QRCodeSVG } from "qrcode.react";
import { useEffect, useMemo, useRef } from "react";

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
  const dialogRef = useRef<HTMLDialogElement>(null);

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

  useEffect(() => {
    dialogRef.current?.showModal();
  }, []);

  return (
    <dialog
      ref={dialogRef}
      className="dialog"
      onClose={onClose}
      onClick={(event) => {
        if (event.target === dialogRef.current) {
          dialogRef.current?.close();
        }
      }}
    >
      <div className="dialog__content">
        <div className="dialog__header">
          <h2 className="dialog__title">Set up device</h2>
          <p className="dialog__description">
            Scan this QR code with the device you want to connect.
          </p>
        </div>

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
            onClick={() => dialogRef.current?.close()}
          >
            Done
          </button>
        </div>
      </div>
    </dialog>
  );
}
