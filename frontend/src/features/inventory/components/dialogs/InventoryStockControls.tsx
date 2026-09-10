import { useState } from "react";
import { useToast } from "../../../../components/toast/ToastContext";
import type { InventoryItem } from "../../types";
import {
  decreaseInventoryStock,
  increaseInventoryStock,
  setInventoryStock,
} from "../../api";

type Props = {
  householdId: string;
  item: InventoryItem;
  onChanged: () => Promise<void>;
};

export function InventoryStockControls({
  householdId,
  item,
  onChanged,
}: Props) {
  const { showToast } = useToast();

  const [newStock, setNewStock] = useState("");
  const [isMutating, setIsMutating] = useState(false);

  function runMutation(
    operation: () => Promise<void>,
    successMessage: string,
    errorMessage: string,
  ) {
    setIsMutating(true);

    void operation()
      .then(async () => {
        await onChanged();
        showToast(successMessage, "success");
      })
      .catch(() => showToast(errorMessage, "error"))
      .finally(() => setIsMutating(false));
  }

  function handleIncreaseStock() {
    runMutation(
      () => increaseInventoryStock(householdId, item.id, { amount: 1 }),
      "Stock increased",
      "Failed to increase stock",
    );
  }

  function handleDecreaseStock() {
    runMutation(
      () => decreaseInventoryStock(householdId, item.id, { amount: 1 }),
      "Stock decreased",
      "Failed to decrease stock",
    );
  }

  function handleSetStock() {
    if (newStock === "") return;

    runMutation(
      async () => {
        await setInventoryStock(householdId, item.id, {
          stock: Number(newStock),
        });

        setNewStock("");
      },
      "Stock updated",
      "Failed to set stock",
    );
  }

  return (
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
  );
}
