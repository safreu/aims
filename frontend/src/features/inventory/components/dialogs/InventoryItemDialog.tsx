import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import Skeleton from "react-loading-skeleton";

import { queryKeys } from "../../../../api/queryKeys";
import { Dialog } from "../../../../components/dialog/Dialog";
import { useTrackingMode } from "../../../accounts/components/tracking-mode/TrackingModeContext";
import { useDangerMode } from "../../../households/danger-mode/DangerModeContext";
import { InventoryItemQrDialog } from "../../../scanning/components/InventoryItemQrDialog";
import { getInventoryItem } from "../../api";
import { InventoryStockHistory } from "../history/InventoryStockHistory";
import { InventoryItemArchive } from "./InventoryItemArchive";
import { InventoryItemDetails } from "./InventoryItemDetails";
import { InventoryStockControls } from "./InventoryStockControls";

import styles from "./InventoryItemDialog.module.css";

type Props = {
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
}: Props) {
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
    <>
      <Dialog
        title={item?.name ?? "Inventory item"}
        description="Manage item details and stock"
        onClose={onClose}
        className={styles.dialog}
      >
        {isPending ? (
          <InventoryItemDialogSkeleton />
        ) : isError ? (
          <p className={styles.error} role="alert">
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

            {trackingMode === "qr" && (
              <section className="dialog__section">
                <div className="dialog__section-header">
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
            )}

            <section className="dialog__section">
              <div className="dialog__section-header">
                <div>
                  <h3>History</h3>
                  <p>Recent changes to this item's stock</p>
                </div>

                <button
                  type="button"
                  className="button button--secondary"
                  onClick={() => setShowHistory((current) => !current)}
                  aria-expanded={showHistory}
                >
                  {showHistory ? "Hide history" : "Show history"}
                </button>
              </div>

              {showHistory && (
                <div className={styles.history}>
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
              onArchived={onClose}
            />
          </>
        )}
      </Dialog>

      {showQrCodes && item && (
        <InventoryItemQrDialog
          householdId={householdId}
          itemId={itemId}
          itemName={item.name}
          onClose={() => setShowQrCodes(false)}
        />
      )}
    </>
  );
}

function InventoryItemDialogSkeleton() {
  return (
    <section className="dialog__section">
      <h3>Details</h3>

      <div className="dialog__fields">
        {Array.from({ length: 4 }).map((_, index) => (
          <div key={index} className="dialog__field">
            <Skeleton width="6rem" height="0.85rem" />

            <Skeleton
              height="var(--control-height)"
              borderRadius="var(--radius-sm)"
            />
          </div>
        ))}
      </div>

      <Skeleton
        width="8rem"
        height="var(--control-height)"
        borderRadius="var(--radius-sm)"
      />
    </section>
  );
}
