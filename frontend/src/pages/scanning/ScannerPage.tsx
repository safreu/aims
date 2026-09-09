import { useRef, useState } from "react";
import "./ScannerPage.css";
import { useParams } from "react-router-dom";
import { getLocalDeviceCredentials } from "../../features/devices/localDevice";
import { executeQrAction } from "../../features/scanning/api";
import { QrScanner } from "../../features/scanning/components/scanner/QrScanner";
import { ApiError } from "../../api/client";

const SAME_QR_SUPPRESSION_MS = 1500;

type ScanStatus =
  | { type: "idle" }
  | { type: "success"; message: string }
  | { type: "error"; message: string };

export function ScannerPage() {
  const { householdId } = useParams();

  if (householdId === undefined) {
    throw new Error("ScannerPage requires a householdId");
  }

  const resolvedHouseholdId = householdId;

  const localCredential = getLocalDeviceCredentials(resolvedHouseholdId);

  const lastScanRef = useRef<{
    value: string;
    scannedAt: number;
  } | null>(null);

  const [isProcessing, setIsProcessing] = useState(false);

  const [scanStatus, setScanStatus] = useState<ScanStatus>({ type: "idle" });

  const [scannerError, setScannerError] = useState<string | null>(null);

  function handleScannerError(error: Error) {
    setScannerError(error.message);
  }

  function handleScan(value: string) {
    if (isProcessing) return;

    const now = Date.now();
    const lastScan = lastScanRef.current;

    if (
      lastScan !== null &&
      lastScan.value === value &&
      now - lastScan.scannedAt < SAME_QR_SUPPRESSION_MS
    )
      return;

    lastScanRef.current = {
      value,
      scannedAt: now,
    };

    if (localCredential === null) return;

    if (!isUUid(value)) {
      setScanStatus({
        type: "error",
        message: "This is not a valid inventory QR code",
      });
      return;
    }

    setScanStatus({ type: "idle" });
    setScannerError(null);
    setIsProcessing(true);

    void executeQrAction(value, localCredential.token)
      .then(() => {
        setScanStatus({ type: "success", message: "Stock updated" });
      })
      .catch((error: unknown) => {
        if (error instanceof ApiError) {
          switch (error.status) {
            case 400:
              setScanStatus({
                type: "error",
                message: "Invalid QR action",
              });
              return;

            case 401:
              setScanStatus({
                type: "error",
                message: "This device is no longer authenticated",
              });
              return;

            case 404:
              setScanStatus({
                type: "error",
                message: "QR action not found",
              });
              return;
          }
        }

        setScanStatus({
          type: "error",
          message: "Failed to execute QR action",
        });
      })
      .finally(() => setIsProcessing(false));
  }

  return (
    <div className="scanner-page">
      <div className="scanner-page__header">
        <h1>Scan QR code</h1>
        <p>Scan an inventory QR code to update stock</p>
      </div>

      {localCredential === null ? (
        <div className="scanner-page__missing-device">
          This device is not registered for this household
        </div>
      ) : (
        <>
          <QrScanner
            paused={isProcessing}
            onScan={handleScan}
            onError={handleScannerError}
          />

          {scannerError !== null && (
            <div className="scanner-page__status scanner-page__status--error">
              {scannerError}
            </div>
          )}

          {scanStatus.type !== "idle" && (
            <div
              className={`scanner-page__status scanner-page__status--${scanStatus.type}`}
            >
              {scanStatus.message}
            </div>
          )}

          {isProcessing && (
            <div className="scanner-page__status">Processing scan...</div>
          )}
        </>
      )}
    </div>
  );
}

function isUUid(value: string): boolean {
  return /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(
    value,
  );
}
