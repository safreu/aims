import { useRef, useState } from "react";
import "./ScannerPage.css";
import { useParams } from "react-router-dom";
import {
  getLocalDeviceCredentials,
  saveLocalDeviceCredential,
} from "../../features/devices/localDevice";
import { executeQrAction } from "../../features/scanning/api";
import { QrScanner } from "../../features/scanning/components/scanner/QrScanner";
import { ApiError } from "../../api/client";
import { useToast } from "../../components/toast/ToastContext";
import {
  issueDeviceCredential,
  registerDevice,
} from "../../features/devices/api";
import { RegisterCurrentDeviceDialog } from "../../features/devices/components/Dialogs/RegisterCurrentDeviceDialog";

const SAME_QR_SUPPRESSION_MS = 1500;

type ScanStatus =
  | { type: "idle" }
  | { type: "success"; message: string }
  | { type: "error"; message: string };

type ScanMode = "single" | "rapid";
const SCAN_MODE_STORAGE_KEY = "scanner-mode";

export function ScannerPage() {
  const { householdId } = useParams();
  const { showToast } = useToast();
  if (householdId === undefined) {
    throw new Error("ScannerPage requires a householdId");
  }

  const resolvedHouseholdId = householdId;

  const [localCredential, setLocalCredential] = useState(() =>
    getLocalDeviceCredentials(resolvedHouseholdId),
  );

  const [showRegisterDeviceDialog, setShowRegisterDeviceDialog] =
    useState(false);

  const [isRegistering, setIsRegistering] = useState(false);

  const lastScanRef = useRef<{
    value: string;
    scannedAt: number;
  } | null>(null);

  const [isProcessing, setIsProcessing] = useState(false);

  const [scanStatus, setScanStatus] = useState<ScanStatus>({ type: "idle" });

  const [scanMode, setScanMode] = useState<ScanMode>(getStoredScanMode);

  const [scannerError, setScannerError] = useState<string | null>(null);

  function handleScanModeChange(mode: ScanMode) {
    setScanMode(mode);
    localStorage.setItem(SCAN_MODE_STORAGE_KEY, mode);

    lastScanRef.current = null;
  }

  async function handleRegisterCurrentDevice(name: string) {
    let deviceId: string;

    setIsRegistering(true);

    return registerDevice(resolvedHouseholdId, { name, kind: "smartphone" })
      .then((registered) => {
        deviceId = registered.id;

        return issueDeviceCredential(resolvedHouseholdId, registered.id);
      })
      .then((credential) => {
        const localCredential = { deviceId, token: credential.token };

        saveLocalDeviceCredential(resolvedHouseholdId, localCredential);
        setLocalCredential(localCredential);

        setShowRegisterDeviceDialog(false);
        showToast("Device registered", "success");
      })
      .catch(() => showToast("Failed to register device", "error"))
      .finally(() => setIsRegistering(false));
  }

  function handleScannerError(error: Error) {
    setScannerError(error.message);
  }

  function handleScan(value: string) {
    if (isProcessing) return;

    const now = Date.now();
    const lastScan = lastScanRef.current;

    if (lastScan !== null && lastScan.value === value) {
      if (scanMode === "single") return;

      if (
        scanMode === "rapid" &&
        now - lastScan.scannedAt < SAME_QR_SUPPRESSION_MS
      )
        return;
    }

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
          <div className="scanner-page__missing-device-info">
            <strong>Register this device</strong>
            <p>
              This phone needs to be registered before it can scan inventory QR
              codes.
            </p>
          </div>

          <button
            type="button"
            className="button button--primary"
            onClick={() => setShowRegisterDeviceDialog(true)}
          >
            Register device
          </button>
        </div>
      ) : (
        <>
          <div
            className="scanner-page__mode"
            role="group"
            aria-label="Scan mode"
          >
            <button
              type="button"
              className={`scanner-page__mode-option${scanMode === "single" ? " active" : ""}`}
              onClick={() => handleScanModeChange("single")}
            >
              Single
            </button>

            <button
              type="button"
              className={`scanner-page__mode-option${scanMode === "rapid" ? " active" : ""}`}
              onClick={() => handleScanModeChange("rapid")}
            >
              Rapid
            </button>
          </div>

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

      {showRegisterDeviceDialog && (
        <RegisterCurrentDeviceDialog
          registering={isRegistering}
          onRegister={handleRegisterCurrentDevice}
          onClose={() => setShowRegisterDeviceDialog(false)}
        />
      )}
    </div>
  );
}

function isUUid(value: string): boolean {
  return /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(
    value,
  );
}

function getStoredScanMode(): ScanMode {
  const stored = localStorage.getItem(SCAN_MODE_STORAGE_KEY);

  return stored === "rapid" ? "rapid" : "single";
}
