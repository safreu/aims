import { useEffect, useMemo, useRef } from "react";
import "./ProvisioningDeviceDialog.css";
import { QRCodeSVG } from "qrcode.react";

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
      className="provision-device-dialog"
      onClose={onClose}
      onClick={(event) => {
        if (event.target === dialogRef.current) {
          dialogRef.current?.close();
        }
      }}
    >
      <div className="provision-device-dialog__content">
        <div className="provision-device-dialog__header">
          <h2>Set up device</h2>
          <p>Scan this QR code with the device you want to connect.</p>
        </div>

        <div className="provision-device-dialog__device">
          <span className="provision-device-dialog__device-label">Device</span>

          <strong>{deviceName}</strong>
        </div>

        <div className="provision-device-dialog__qr">
          <QRCodeSVG value={qrValue} size={220} level="M" />
        </div>

        <p className="provision-device-dialog__hint">
          Keep this screen open until the device has completed setup
        </p>

        <div className="provision-device-dialog__actions">
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
