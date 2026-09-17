import { useQuery } from "@tanstack/react-query";
import { X } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import Skeleton from "react-loading-skeleton";
import { QRCodeCanvas, QRCodeSVG } from "qrcode.react";

import { queryKeys } from "../../../api/queryKeys";
import { useToast } from "../../../components/toast/ToastContext";
import { getQrActions } from "../api";
import type { QrActionKind } from "../types";
import { shareQrCard } from "../utils/qrShare";

import styles from "./InventoryItemQrDialog.module.css";

type Props = {
  householdId: string;
  itemId: string;
  itemName: string;
  onClose: () => void;
};

export function InventoryItemQrDialog({
  householdId,
  itemId,
  itemName,
  onClose,
}: Props) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const qrCanvasRef = useRef<HTMLCanvasElement>(null);

  const { showToast } = useToast();

  const {
    data: qrActions = [],
    isPending,
    isError,
  } = useQuery({
    queryKey: queryKeys.inventory.qrActions(householdId, itemId),
    queryFn: async () => {
      const actions = await getQrActions(householdId);

      return actions.filter((action) => action.item_id === itemId);
    },
  });

  const [selectedKind, setSelectedKind] = useState<QrActionKind>("increase");
  const [isSharing, setIsSharing] = useState(false);

  const selectedAction = useMemo(
    () => qrActions.find((action) => action.kind === selectedKind) ?? null,
    [qrActions, selectedKind],
  );

  useEffect(() => {
    dialogRef.current?.showModal();
  }, []);

  async function handleShare() {
    if (selectedAction === null || qrCanvasRef.current === null) {
      return;
    }

    setIsSharing(true);

    try {
      const result = await shareQrCard(
        qrCanvasRef.current,
        itemName,
        selectedAction,
      );

      if (result === "downloaded") {
        showToast("QR code downloaded", "success");
      }
    } catch {
      showToast("Failed to share QR code", "error");
    } finally {
      setIsSharing(false);
    }
  }

  function selectKind(kind: QrActionKind) {
    if (isSharing) return;

    setSelectedKind(kind);
  }

  return (
    <dialog
      ref={dialogRef}
      className={`dialog ${styles.dialog}`}
      onClose={onClose}
      onCancel={(event) => {
        if (isSharing) {
          event.preventDefault();
        }
      }}
      onClick={(event) => {
        if (event.target === dialogRef.current && !isSharing) {
          dialogRef.current?.close();
        }
      }}
    >
      <div className="dialog__content">
        <header className="dialog__header">
          <div>
            <h2 className="dialog__title">{itemName}</h2>

            <p className="dialog__description">Inventory QR code</p>
          </div>

          <button
            type="button"
            className="dialog__close"
            onClick={() => dialogRef.current?.close()}
            disabled={isSharing}
            aria-label="Close"
          >
            <X aria-hidden="true" />
          </button>
        </header>

        <div
          className={styles.switcher}
          role="group"
          aria-label="QR code action"
        >
          <button
            type="button"
            className={`${styles.option} ${
              selectedKind === "decrease" ? styles.optionActive : ""
            }`}
            onClick={() => selectKind("decrease")}
            disabled={isSharing}
            aria-pressed={selectedKind === "decrease"}
          >
            Decrease
          </button>

          <button
            type="button"
            className={`${styles.option} ${
              selectedKind === "increase" ? styles.optionActive : ""
            }`}
            onClick={() => selectKind("increase")}
            disabled={isSharing}
            aria-pressed={selectedKind === "increase"}
          >
            Increase
          </button>
        </div>

        {isPending ? (
          <InventoryItemQrSkeleton />
        ) : isError ? (
          <p className={styles.error} role="alert">
            Failed to load QR codes
          </p>
        ) : selectedAction === null ? (
          <p className={styles.error}>QR code not available</p>
        ) : (
          <div className={styles.qr}>
            <div className={styles.qrCard}>
              <QRCodeSVG value={selectedAction.id} size={220} level="M" />
            </div>

            <button
              type="button"
              className="button button--secondary"
              onClick={() => void handleShare()}
              disabled={isSharing}
            >
              {isSharing ? "Preparing..." : "Share QR code"}
            </button>

            <div className={styles.exportQr} aria-hidden="true">
              <QRCodeCanvas
                ref={qrCanvasRef}
                value={selectedAction.id}
                size={512}
                level="M"
              />
            </div>
          </div>
        )}
      </div>
    </dialog>
  );
}

function InventoryItemQrSkeleton() {
  return (
    <div className={styles.qr} aria-label="Loading QR code">
      <div className={styles.qrPlaceholder}>
        <Skeleton width={200} height={200} borderRadius="var(--radius-md)" />
      </div>

      <Skeleton width="8rem" height={40} borderRadius="var(--radius-md)" />
    </div>
  );
}
