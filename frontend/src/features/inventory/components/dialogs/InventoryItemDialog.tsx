import { useEffect, useRef, useState, type SubmitEvent } from "react";
import { type InventoryItemPriority, type InventoryItem } from "../../types";
import {
  archiveInventoryItem,
  decreaseInventoryStock,
  getInventoryItem,
  increaseInventoryStock,
  setInventoryStock,
  updateInventoryItem,
} from "../../api";
import { InventoryStockHistory } from "../history/InventoryStockHistory";
import "./InventoryItemDialog.css";
import { InventoryItemFields } from "../fields/InventoryItemFields";

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

  const [item, setItem] = useState<InventoryItem | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);

  const [name, setName] = useState("");
  const [categoryId, setCategoryId] = useState<string | null>(null);
  const [reorderThreshold, setReorderThreshold] = useState(0);
  const [priority, setPriority] = useState<InventoryItemPriority>("default");
  const [newStock, setNewStock] = useState("");
  const [showHistory, setShowHistory] = useState(false);
  const [historyVersion, setHistoryVersion] = useState(0);

  const [isMutating, setIsMutating] = useState(false);
  const [mutationError, setMutationError] = useState<string | null>(null);

  async function refreshItem() {
    const refreshedItem = await getInventoryItem(householdId, itemId);

    setItem(refreshedItem);
    setName(refreshedItem.name);
    setCategoryId(refreshedItem.category?.id ?? null);
    setReorderThreshold(refreshedItem.reorder_threshold);
    setPriority(refreshedItem.priority);
  }

  function handleClose() {
    void onChanged().finally(() => onClose());
  }

  useEffect(() => {
    dialogRef.current?.showModal();

    async function loadItem() {
      await getInventoryItem(householdId, itemId)
        .then((loadedItem) => {
          setItem(loadedItem);
          setName(loadedItem.name);
          setCategoryId(loadedItem.category?.id ?? null);
          setReorderThreshold(loadedItem.reorder_threshold);
          setPriority(loadedItem.priority);
        })
        .catch(() => setLoadError("Failed to load inventory item"))
        .finally(() => setLoading(false));
    }

    void loadItem();
  }, [householdId, itemId]);

  function runMutation(operation: () => Promise<void>) {
    setIsMutating(true);
    setMutationError(null);

    void operation()
      .then(async () => {
        await refreshItem();
        await onChanged();
      })
      .catch(() => setMutationError("Failed to mutate item"))
      .finally(() => setIsMutating(false));
  }

  function handleIncreaseStock() {
    runMutation(async () => {
      await increaseInventoryStock(householdId, itemId, { amount: 1 });

      setHistoryVersion((current) => current + 1);
    });
  }

  function handleDecreaseStock() {
    runMutation(async () => {
      await decreaseInventoryStock(householdId, itemId, { amount: 1 });
      setHistoryVersion((current) => current + 1);
    });
  }

  function handleSetStock() {
    if (newStock === "") return;

    runMutation(async () => {
      await setInventoryStock(householdId, itemId, {
        stock: Number(newStock),
      });
      setNewStock("");
      setHistoryVersion((current) => current + 1);
    });
  }

  function handleArchive() {
    void archiveInventoryItem(householdId, itemId)
      .then(() => dialogRef.current?.close())
      .catch(() => setMutationError("Failed to archive item"))
      .finally(() => setIsMutating(false));
  }

  function handleUpdate(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    runMutation(async () => {
      await updateInventoryItem(householdId, itemId, {
        name,
        category_id: categoryId,
        reorder_threshold: reorderThreshold,
        priority,
      });
    });
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

        {loading ? (
          <p>Loading item...</p>
        ) : loadError !== null ? (
          <p className="inventory-item-dialog__error">{loadError}</p>
        ) : item !== null ? (
          <>
            <form
              className="inventory-item-dialog__section"
              onSubmit={handleUpdate}
            >
              <h3>Details</h3>

              <InventoryItemFields
                householdId={householdId}
                name={name}
                categoryId={categoryId}
                reorderThreshold={reorderThreshold}
                priority={priority}
                onNameChange={setName}
                onReorderThresholdChange={setReorderThreshold}
                onPriorityChange={setPriority}
                onCategoryChange={setCategoryId}
                disabled={isMutating}
              />

              <button
                type="submit"
                className="button button--primary"
                disabled={isMutating}
              >
                {isMutating ? "Saving..." : "Save changes"}
              </button>
            </form>

            <section className="inventory-item-dialog__section">
              <h3>Stock</h3>

              <div className="inventory-item-dialog__stock-controls">
                <button
                  type="button"
                  className="inventory-item-dialog__stock-button"
                  onClick={handleDecreaseStock}
                  disabled={item.current_stock === 0 || isMutating}
                  aria-label="Decrease stock by one"
                >
                  -
                </button>

                <button
                  type="button"
                  className="inventory-item-dialog__stock-button"
                  onClick={handleIncreaseStock}
                  disabled={isMutating}
                  aria-label="Increase stock by one"
                >
                  +
                </button>
              </div>

              <div className="inventory-item-dialog__set-stock">
                <label className="inventory-item-dialog__field">
                  <span>Set exact stock</span>
                </label>

                <div className="inventory-item-dialog__set-stock-controls">
                  <input
                    type="number"
                    min="0"
                    placeholder={String(item.current_stock)}
                    value={newStock}
                    onChange={(event) => setNewStock(event.target.value)}
                  />

                  <button
                    type="button"
                    className="button button--primary"
                    onClick={handleSetStock}
                    disabled={isMutating || newStock === ""}
                  >
                    Set stock
                  </button>
                </div>
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
                disabled={isMutating}
                onClick={() => setShowHistory((current) => !current)}
              >
                {showHistory ? "Hide history" : "Show history"}
              </button>

              {showHistory && (
                <div className="inventory-item-dialog__history">
                  <InventoryStockHistory
                    householdId={householdId}
                    itemId={itemId}
                    version={historyVersion}
                  />
                </div>
              )}
            </section>

            <section className="inventory-item-dialog__section inventory-item-dialog__danger">
              <h3>Archive item</h3>
              <p>The item will disappear from the active inventory</p>

              <button
                type="button"
                className="button button--danger"
                onClick={handleArchive}
                disabled={isMutating}
              >
                Archive item
              </button>
            </section>

            {mutationError !== null && (
              <p className="inventory-item-dialog__error">{mutationError}</p>
            )}
          </>
        ) : null}
      </div>
    </dialog>
  );
}
