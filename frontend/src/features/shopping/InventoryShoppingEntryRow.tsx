import { useEffect, useState } from "react";
import type { InventoryShoppingEntry } from "./types";
import {
  dismissShoppingItem,
  setShoppingChecked,
  setShoppingNote,
  setShoppingQuantity,
} from "./api";

type InventoryShoppingEntryRowProps = {
  householdId: string;
  entry: InventoryShoppingEntry;
  onChange: () => Promise<void>;
};

export function InventoryShoppingEntryRow({
  householdId,
  entry,
  onChange,
}: InventoryShoppingEntryRowProps) {
  const [quantity, setQuantity] = useState(entry.quantity);
  const [note, setNote] = useState(entry.note ?? "");
  const [isMutating, setIsMutating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setQuantity(entry.quantity);
    setNote(entry.name ?? "");
  }, [entry.quantity, entry.note]);

  function runMutation(operation: () => Promise<void>) {
    setIsMutating(true);
    setError(null);

    void operation()
      .catch(() => setError("Failed to update shopping list"))
      .finally(() => setIsMutating(false));
  }

  function handleCheckedUpdate(checked: boolean) {
    runMutation(async () => {
      await setShoppingChecked(householdId, entry.item_id, { checked });
      await onChange();
    });
  }

  function handleQuantityUpdate() {
    runMutation(async () => {
      await setShoppingQuantity(householdId, entry.item_id, { quantity });
      await onChange();
    });
  }
  function handleNoteUpdate() {
    runMutation(async () => {
      await setShoppingNote(householdId, entry.item_id, {
        note: note.trim() === "" ? null : note.trim(),
      });
      await onChange();
    });
  }

  function handleDismiss() {
    runMutation(async () => {
      await dismissShoppingItem(householdId, entry.item_id);
      await onChange();
    });
  }

  return (
    <li>
      <div>
        <input
          type="checkbox"
          checked={entry.checked}
          disabled={isMutating}
          onChange={(event) => handleCheckedUpdate(event.target.checked)}
        />

        <strong>{entry.name}</strong>

        <span>Priority: {entry.priority}</span>

        {entry.category !== null && (
          <span>Category: {entry.category.name}</span>
        )}
      </div>

      <div>
        <input
          type="number"
          min="1"
          value={quantity}
          disabled={isMutating}
          onChange={(event) => setQuantity(Number(event.target.value))}
        />

        <button
          type="button"
          disabled={isMutating}
          onClick={handleQuantityUpdate}
        >
          Save quantity
        </button>
      </div>

      <div>
        <input
          type="text"
          value={note}
          placeholder={"Note"}
          disabled={isMutating}
          onChange={(event) => setNote(event.target.value)}
        />

        <button type="button" disabled={isMutating} onClick={handleNoteUpdate}>
          Save note
        </button>
      </div>

      <button type="button" disabled={isMutating} onClick={handleDismiss}>
        Dismiss
      </button>

      {error !== null && <p>{error}</p>}
    </li>
  );
}
