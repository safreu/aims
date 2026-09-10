import { useEffect, useMemo, useRef, useState } from "react";
import "./InventoryItemQrDialog.css";
import type { QrActionKind } from "../types";
import { getQrActions } from "../api";
import { useToast } from "../../../components/toast/ToastContext";
import { QRCodeCanvas, QRCodeSVG } from "qrcode.react";
import { shareQrCard } from "../utils/qrShare";
import { useQuery } from "@tanstack/react-query";
import { queryKeys } from "../../../api/queryKeys";
import Skeleton from "react-loading-skeleton";

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

  async function handleShare() {
    if (selectedAction === null || qrCanvasRef.current === null) return;

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

  useEffect(() => {
    dialogRef.current?.showModal();
  }, []);

  return (
    <dialog
      ref={dialogRef}
      className="inventory-item-qr-dialog"
      onClose={onClose}
      onClick={(event) => {
        if (event.target === dialogRef.current) {
          dialogRef.current?.close();
        }
      }}
    >
      <div className="inventory-item-qr-dialog__content">
        <header className="inventory-item-qr-dialog__header">
          <div>
            <h2>{itemName}</h2>
            <p>Inventory QR code</p>
          </div>

          <button
            type="button"
            className="button button--ghost"
            onClick={() => dialogRef.current?.close()}
          >
            Close
          </button>
        </header>

        <div className="inventory-item-qr-dialog__switcher">
          <button
            type="button"
            className={`inventory-item-qr-dialog__option ${
              selectedKind === "decrease"
                ? "inventory-item-qr-dialog__option--active"
                : ""
            }`}
            onClick={() => setSelectedKind("decrease")}
          >
            Decrease
          </button>

          <button
            type="button"
            className={`inventory-item-qr-dialog__option ${
              selectedKind === "increase"
                ? "inventory-item-qr-dialog__option--active"
                : ""
            }`}
            onClick={() => setSelectedKind("increase")}
          >
            Increase
          </button>
        </div>

        {isPending ? (
          <InventoryItemQrSkeleton />
        ) : isError ? (
          <p className="inventory-item-qr-dialog__error">
            Failed to load QR codes
          </p>
        ) : selectedAction === null ? (
          <p className="inventory-item-qr-dialog__error">
            QR code not available
          </p>
        ) : (
          <div className="inventory-item-qr-dialog__qr">
            <div className="inventory-item-qr-dialog__qr-card">
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

            <div className="inventory-item-qr-dialog__export-qr">
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
    <div className="inventory-item-qr-dialog__qr">
      <div className="inventory-item-qr-dialog__qr-placeholder">
        <Skeleton width={200} height={200} borderRadius="var(--radius-md)" />
      </div>

      <Skeleton width="8rem" height={40} borderRadius="var(--radius-md)" />
    </div>
  );
}
