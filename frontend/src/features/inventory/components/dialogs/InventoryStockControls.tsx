import { useState } from "react";

import { useToast } from "../../../../components/toast/ToastContext";
import {
  decreaseInventoryStock,
  increaseInventoryStock,
  setInventoryStock,
} from "../../api";
import type { InventoryItem } from "../../types";

import styles from "./InventoryStockControls.module.css";

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
      () =>
        increaseInventoryStock(householdId, item.id, {
          amount: 1,
        }),
      "Stock increased",
      "Failed to increase stock",
    );
  }

  function handleDecreaseStock() {
    runMutation(
      () =>
        decreaseInventoryStock(householdId, item.id, {
          amount: 1,
        }),
      "Stock decreased",
      "Failed to decrease stock",
    );
  }

  function handleSetStock() {
    if (newStock === "") return;

    const stock = Number(newStock);

    if (!Number.isFinite(stock) || stock < 0) return;

    runMutation(
      async () => {
        await setInventoryStock(householdId, item.id, {
          stock,
        });

        setNewStock("");
      },
      "Stock updated",
      "Failed to set stock",
    );
  }

  const parsedStock = Number(newStock);

  const canSetStock =
    newStock !== "" && Number.isFinite(parsedStock) && parsedStock >= 0;

  return (
    <section className="dialog__section">
      <h3>Stock</h3>

      <div className={styles.controls}>
        <button
          type="button"
          className={styles.stockButton}
          onClick={handleDecreaseStock}
          disabled={item.current_stock === 0 || isMutating}
          aria-label="Decrease stock by one"
        >
          −
        </button>

        <button
          type="button"
          className={styles.stockButton}
          onClick={handleIncreaseStock}
          disabled={isMutating}
          aria-label="Increase stock by one"
        >
          +
        </button>
      </div>

      <div className={styles.setStock}>
        <label className="dialog__field">
          <span>Set exact stock</span>

          <div className={styles.setStockControls}>
            <input
              type="number"
              min="0"
              step="1"
              placeholder={String(item.current_stock)}
              value={newStock}
              onChange={(event) => setNewStock(event.target.value)}
              disabled={isMutating}
            />

            <button
              type="button"
              className="button button--primary"
              onClick={handleSetStock}
              disabled={isMutating || !canSetStock}
            >
              {isMutating ? "Updating..." : "Set stock"}
            </button>
          </div>
        </label>
      </div>
    </section>
  );
}
