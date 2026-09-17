import { useRef, useState } from "react";
import { useParams } from "react-router-dom";

import { ApiError } from "../../../api/client";
import { useToast } from "../../../components/toast/ToastContext";
import { issueDeviceCredential, registerDevice } from "../../devices/api";
import { RegisterCurrentDeviceDialog } from "../../devices/components/dialogs/RegisterCurrentDeviceDialog";
import {
  getLocalDeviceCredentials,
  saveLocalDeviceCredential,
} from "../../devices/localDevice";
import { executeQrAction } from "../api";
import { QrScanner } from "../components/scanner/QrScanner";

import styles from "./ScannerPage.module.css";

const SAME_QR_SUPPRESSION_MS = 1500;
const SCAN_MODE_STORAGE_KEY = "scanner-mode";

type ScanStatus =
  | { type: "idle" }
  | { type: "success"; message: string }
  | { type: "error"; message: string };

type ScanMode = "single" | "rapid";

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

  const [isProcessing, setIsProcessing] = useState(false);

  const [scanStatus, setScanStatus] = useState<ScanStatus>({ type: "idle" });

  const [scanMode, setScanMode] = useState<ScanMode>(getStoredScanMode);

  const [scannerError, setScannerError] = useState<string | null>(null);

  const lastScanRef = useRef<{
    value: string;
    scannedAt: number;
  } | null>(null);

  function handleScanModeChange(mode: ScanMode) {
    setScanMode(mode);
    localStorage.setItem(SCAN_MODE_STORAGE_KEY, mode);

    lastScanRef.current = null;
  }

  async function handleRegisterCurrentDevice(name: string) {
    let deviceId: string;

    setIsRegistering(true);

    return registerDevice(resolvedHouseholdId, {
      name,
      kind: "smartphone",
    })
      .then((registered) => {
        deviceId = registered.id;

        return issueDeviceCredential(resolvedHouseholdId, registered.id);
      })
      .then((credential) => {
        const localCredential = {
          deviceId,
          token: credential.token,
        };

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
    if (isProcessing || localCredential === null) {
      return;
    }

    if (!isUuid(value)) {
      setScanStatus({
        type: "error",
        message: "This is not a valid inventory QR code",
      });

      return;
    }

    const now = Date.now();
    const lastScan = lastScanRef.current;

    if (lastScan !== null && lastScan.value === value) {
      if (scanMode === "single") {
        return;
      }

      if (
        scanMode === "rapid" &&
        now - lastScan.scannedAt < SAME_QR_SUPPRESSION_MS
      ) {
        return;
      }
    }

    lastScanRef.current = {
      value,
      scannedAt: now,
    };

    setScanStatus({ type: "idle" });
    setScannerError(null);
    setIsProcessing(true);

    void executeQrAction(value, localCredential.token)
      .then(() => {
        setScanStatus({
          type: "success",
          message: "Stock updated",
        });
      })
      .catch((error: unknown) => {
        /*
         * Allow another scan of the same QR after a failed
         * execution. A failed request should not count as a
         * completed single scan.
         */
        if (lastScanRef.current?.value === value) {
          lastScanRef.current = null;
        }

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
    <div className={styles.page}>
      <header className={styles.header}>
        <h1>Scan QR code</h1>
        <p>Scan an inventory QR code to update stock</p>
      </header>

      {localCredential === null ? (
        <div className={styles.missingDevice}>
          <div className={styles.missingDeviceInfo}>
            <strong>Register this device</strong>

            <p>
              This phone needs to be registered before it can scan inventory QR
              codes.
            </p>
          </div>

          <button
            type="button"
            className={`button button--primary ${styles.registerButton}`}
            onClick={() => setShowRegisterDeviceDialog(true)}
          >
            Register device
          </button>
        </div>
      ) : (
        <>
          <div className={styles.mode} role="group" aria-label="Scan mode">
            <button
              type="button"
              className={`${styles.modeOption} ${
                scanMode === "single" ? styles.modeOptionActive : ""
              }`}
              onClick={() => handleScanModeChange("single")}
              aria-pressed={scanMode === "single"}
            >
              Single
            </button>

            <button
              type="button"
              className={`${styles.modeOption} ${
                scanMode === "rapid" ? styles.modeOptionActive : ""
              }`}
              onClick={() => handleScanModeChange("rapid")}
              aria-pressed={scanMode === "rapid"}
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
            <div
              className={`${styles.status} ${styles.statusError}`}
              role="alert"
            >
              {scannerError}
            </div>
          )}

          {scanStatus.type !== "idle" && (
            <div
              className={`${styles.status} ${
                scanStatus.type === "success"
                  ? styles.statusSuccess
                  : styles.statusError
              }`}
              role={scanStatus.type === "error" ? "alert" : "status"}
            >
              {scanStatus.message}
            </div>
          )}

          {isProcessing && (
            <div className={styles.status} role="status">
              Processing scan...
            </div>
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

function isUuid(value: string): boolean {
  return /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(
    value,
  );
}

function getStoredScanMode(): ScanMode {
  const stored = localStorage.getItem(SCAN_MODE_STORAGE_KEY);

  return stored === "rapid" ? "rapid" : "single";
}
