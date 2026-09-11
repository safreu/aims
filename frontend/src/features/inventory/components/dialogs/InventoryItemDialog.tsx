import { useEffect, useRef, useState } from "react";
import { getInventoryItem } from "../../api";
import { InventoryStockHistory } from "../history/InventoryStockHistory";
import "./InventoryItemDialog.css";
import { useDangerMode } from "../../../households/danger-mode/DangerModeContext";
import { useTrackingMode } from "../../../accounts/components/tracking-mode/TrackingModeContext";
import { InventoryItemQrDialog } from "../../../scanning/components/InventoryItemQrDialog";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { queryKeys } from "../../../../api/queryKeys";
import Skeleton from "react-loading-skeleton";
import { InventoryItemDetails } from "./InventoryItemDetails";
import { InventoryStockControls } from "./InventoryStockControls";
import { InventoryItemArchive } from "./InventoryItemArchive";

type InventoryItemDialogProps = {
  householdId: string;
  itemId: string;
  onChanged: () => Promise<void>;
  onClose: () => void;
};

export function InventoryItemDialog({
  householdId,
  itemId,
  onChanged,
  onClose,
}: InventoryItemDialogProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);

  const { dangerMode } = useDangerMode();
  const { trackingMode } = useTrackingMode();

  const queryClient = useQueryClient();

  const {
    data: item,
    isPending,
    isError,
  } = useQuery({
    queryKey: queryKeys.inventory.item(householdId, itemId),
    queryFn: () => getInventoryItem(householdId, itemId),
  });

  const [showHistory, setShowHistory] = useState(false);

  const [showQrCodes, setShowQrCodes] = useState(false);

  const showStockControls = dangerMode || trackingMode === "manual";

  function handleClose() {
    void onChanged().finally(() => onClose());
  }

  useEffect(() => {
    dialogRef.current?.showModal();
  }, []);

  async function refreshItem() {
    await Promise.all([
      queryClient.invalidateQueries({
        queryKey: queryKeys.inventory.item(householdId, itemId),
      }),
      onChanged(),
    ]);
  }

  async function refreshStock() {
    await Promise.all([
      queryClient.invalidateQueries({
        queryKey: queryKeys.inventory.item(householdId, itemId),
      }),
      queryClient.invalidateQueries({
        queryKey: queryKeys.inventory.history(householdId, itemId),
      }),
      onChanged(),
    ]);
  }

  return (
    <dialog
      ref={dialogRef}
      className="inventory-item-dialog"
      onClose={handleClose}
      onClick={(event) => {
        if (event.target === dialogRef.current) {
          dialogRef.current?.close();
        }
      }}
    >
      <div className="inventory-item-dialog__content">
        <header className="inventory-item-dialog__header">
          <div>
            <h2>{item?.name ?? "Inventory item"}</h2>
            <p>Manage item details and stock</p>
          </div>

          <button
            type="button"
            className="button button--ghost inventory-item-dialog__close"
            onClick={() => dialogRef.current?.close()}
          >
            Close
          </button>
        </header>

        {isPending ? (
          <InventoryItemDialogSkeleton />
        ) : isError ? (
          <p className="inventory-item-dialog__error">
            Failed to load inventory item
          </p>
        ) : (
          <>
            <InventoryItemDetails
              householdId={householdId}
              item={item}
              onChanged={refreshItem}
            />
            {showStockControls && (
              <InventoryStockControls
                householdId={householdId}
                item={item}
                onChanged={refreshStock}
              />
            )}

            <section className="inventory-item-dialog__section">
              <div className="inventory-item-dialog__section-header">
                <div>
                  <h3>QR codes</h3>
                  <p>View or share the QR codes for this item</p>
                </div>

                <button
                  type="button"
                  className="button button--secondary"
                  onClick={() => setShowQrCodes(true)}
                >
                  Show QR codes
                </button>
              </div>
            </section>

            <section className="inventory-item-dialog__section">
              <div className="inventory-item-dialog__section-header">
                <h3>History</h3>
                <p>Recent changes to this item's stock</p>
              </div>

              <button
                type="button"
                className="button button--secondary"
                onClick={() => setShowHistory((current) => !current)}
              >
                {showHistory ? "Hide history" : "Show history"}
              </button>

              {showHistory && (
                <div className="inventory-item-dialog__history">
                  <InventoryStockHistory
                    householdId={householdId}
                    itemId={itemId}
                  />
                </div>
              )}
            </section>

            <InventoryItemArchive
              householdId={householdId}
              item={item}
              onArchived={() => dialogRef.current?.close()}
            />
          </>
        )}

        {showQrCodes && (
          <InventoryItemQrDialog
            householdId={householdId}
            itemId={itemId}
            itemName={item?.name ?? ""}
            onClose={() => setShowQrCodes(false)}
          />
        )}
      </div>
    </dialog>
  );
}

function InventoryItemDialogSkeleton() {
  return (
    <>
      <section className="inventory-item-dialog__section">
        <h3>Details</h3>

        <div className="inventory-item-dialog__fields">
          {Array.from({ length: 4 }).map((_, index) => (
            <div key={index} className="inventory-item-dialog__field">
              <Skeleton width="6rem" height="0.85rem" />
              <Skeleton height={44} borderRadius="var(--radius-sm)" />
            </div>
          ))}
        </div>

        <Skeleton width="8rem" height={40} borderRadius="var(--radius-md)" />
      </section>
    </>
  );
}
