import { useEffect, useMemo, useRef, useState } from "react";
import "./InventoryItemQrDialog.css";
import type { QrAction, QrActionKind } from "../types";
import { getQrActions } from "../api";
import { useToast } from "../../../components/toast/ToastContext";
import { QRCodeCanvas, QRCodeSVG } from "qrcode.react";
import { shareQrCard } from "../utils/qrShare";

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

  const [qrActions, setQrActions] = useState<QrAction[]>([]);
  const [selectedKind, setSelectedKind] = useState<QrActionKind>("increase");

  const [isSharing, setIsSharing] = useState(false);

  const [loading, setLoading] = useState(true);

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

    void getQrActions(householdId)
      .then((actions) => {
        setQrActions(actions.filter((action) => action.item_id === itemId));
      })
      .catch(() => {
        showToast("Failed to fetch QR actions", "error");
      })
      .finally(() => setLoading(false));
  }, [householdId, itemId, showToast]);

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

        {loading ? (
          <p>Loading QR code...</p>
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
