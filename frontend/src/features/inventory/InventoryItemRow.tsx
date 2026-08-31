import { useState, type SubmitEvent } from "react";
import {
  type InventoryItemPriority,
  type InventoryItem,
  type InventoryItemCategory,
} from "./types";
import {
  archiveInventoryItem,
  decreaseInventoryStock,
  increaseInventoryStock,
  setInventoryStock,
  updateInventoryItem,
} from "./api";
import { InventoryStockHistory } from "./InventoryStockHistory";

type InventoryItemRowProps = {
  householdId: string;
  item: InventoryItem;
  categories: InventoryItemCategory[];
  onChanged: () => Promise<void>;
};

export function InventoryItemRow({
  householdId,
  item,
  categories,
  onChanged,
}: InventoryItemRowProps) {
  const [name, setName] = useState(item.name);
  const [categoryId, setCategoryId] = useState(item.category?.id ?? "");
  const [reorderThreshold, setReorderThreshold] = useState(
    item.reorder_threshold,
  );
  const [priority, setPriority] = useState<InventoryItemPriority>(
    item.priority,
  );
  const [newStock, setNewStock] = useState("");
  const [showHistory, setShowHistory] = useState(false);
  const [historyVersion, setHistoryVersion] = useState(0);

  const [isMutating, setIsMutating] = useState(false);
  const [mutationError, setMutationError] = useState<string | null>(null);

  function runMutation(operation: () => Promise<void>) {
    setIsMutating(true);
    setMutationError(null);

    void operation()
      .catch(() => setMutationError("Failed to mutate item"))
      .finally(() => setIsMutating(false));
  }

  function handleIncreaseStock() {
    runMutation(async () => {
      await increaseInventoryStock(householdId, item.id, { amount: 1 }).then(
        () => {
          setHistoryVersion((current) => current + 1);
          return onChanged();
        },
      );
    });
  }

  function handleDecreaseStock() {
    runMutation(async () => {
      await decreaseInventoryStock(householdId, item.id, { amount: 1 }).then(
        () => {
          setHistoryVersion((current) => current + 1);
          return onChanged();
        },
      );
    });
  }

  function handleSetStock() {
    if (newStock === "") return;

    runMutation(async () => {
      await setInventoryStock(householdId, item.id, {
        stock: Number(newStock),
      }).then(() => {
        setNewStock("");
        setHistoryVersion((current) => current + 1);
        return onChanged();
      });
    });
  }

  function handleArchive() {
    void archiveInventoryItem(householdId, item.id).then(() => onChanged());
  }

  function handleUpdate(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    void updateInventoryItem(householdId, item.id, {
      name,
      category_id: categoryId === "" ? null : categoryId,
      reorder_threshold: reorderThreshold,
      priority,
    }).then(() => onChanged());
  }

  return (
    <form onSubmit={handleUpdate}>
      <input
        type="text"
        value={name}
        onChange={(event) => setName(event.target.value)}
      />

      <select
        value={categoryId}
        onChange={(event) => setCategoryId(event.target.value)}
      >
        <option value="">No category</option>

        {categories.map((category) => (
          <option key={category.id} value={category.id}>
            {category.name}
          </option>
        ))}
      </select>

      <input
        type="number"
        min="0"
        value={reorderThreshold}
        onChange={(event) => setReorderThreshold(Number(event.target.value))}
      />

      <select
        value={priority}
        onChange={(event) =>
          setPriority(event.target.value as InventoryItemPriority)
        }
      >
        <option value="default">Default</option>
        <option value="low">Low</option>
        <option value="medium">Medium</option>
        <option value="high">High</option>
      </select>

      <button type="submit" disabled={isMutating}>
        Save
      </button>

      <div>
        <span>Stock: {item.current_stock}</span>
        {mutationError !== null && <p>{mutationError}</p>}

        <button
          type="button"
          onClick={handleDecreaseStock}
          disabled={item.current_stock === 0 || isMutating}
        >
          {" "}
          -1{" "}
        </button>
        <button
          type="button"
          onClick={handleIncreaseStock}
          disabled={isMutating}
        >
          {" "}
          +1{" "}
        </button>
        <input
          type="number"
          min="0"
          placeholder={String(item.current_stock)}
          value={newStock}
          onChange={(event) => setNewStock(event.target.value)}
        />
        <button type="button" onClick={handleSetStock} disabled={isMutating}>
          Set stock
        </button>
      </div>
      <div>
        <button type="button" onClick={handleArchive} disabled={isMutating}>
          Archive
        </button>
      </div>
      <div>
        <button
          type="button"
          disabled={isMutating}
          onClick={() => setShowHistory((current) => !current)}
        >
          {showHistory ? "Hide history" : "Show history"}
        </button>

        {showHistory && (
          <InventoryStockHistory
            householdId={householdId}
            itemId={item.id}
            version={historyVersion}
          />
        )}
      </div>
    </form>
  );
}
