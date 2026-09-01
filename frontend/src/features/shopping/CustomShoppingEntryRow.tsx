import { useEffect, useState, type SubmitEvent } from "react";
import type { CustomShoppingEntry, ShoppingPriority } from "./types";
import {
  deleteCustomShoppingEntry,
  setCustomShoppingChecked,
  updateCustomShoppingEntry,
} from "./api";

type CustomShoppingEntryRowProps = {
  householdId: string;
  entry: CustomShoppingEntry;
  onChange: () => Promise<void>;
};

export function CustomShoppingEntryRow({
  householdId,
  entry,
  onChange,
}: CustomShoppingEntryRowProps) {
  const [title, setTitle] = useState(entry.title);
  const [quantity, setQuantity] = useState(entry.quantity);
  const [priority, setPriority] = useState(entry.priority);
  const [note, setNote] = useState(entry.note ?? "");
  const [isMutating, setIsMutating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setTitle(entry.title);
    setQuantity(entry.quantity);
    setPriority(entry.priority);
    setNote(entry.note ?? "");
  }, [entry.title, entry.quantity, entry.priority, entry.note]);

  function runMutation(operation: () => Promise<void>) {
    setIsMutating(true);
    setError(null);

    void operation()
      .catch(() => setError("Failed to update shopping list"))
      .finally(() => setIsMutating(false));
  }

  function handleUpdate(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    const trimmedTitle = title.trim();

    if (trimmedTitle === "") {
      setError("Title is required");
      return;
    }

    runMutation(async () => {
      await updateCustomShoppingEntry(householdId, entry.id, {
        title: trimmedTitle,
        quantity,
        priority,
        note: note.trim() === "" ? null : note.trim(),
      });
      await onChange();
    });
  }

  function handleCheckedUpdate(checked: boolean) {
    runMutation(async () => {
      await setCustomShoppingChecked(householdId, entry.id, { checked });
      await onChange();
    });
  }

  function handleDelete() {
    runMutation(async () => {
      await deleteCustomShoppingEntry(householdId, entry.id);
      await onChange();
    });
  }

  return (
    <li>
      <form onSubmit={handleUpdate}>
        <input
          type="checkbox"
          checked={entry.checked}
          disabled={isMutating}
          onChange={(event) => handleCheckedUpdate(event.target.checked)}
        />

        <input
          type="text"
          value={title}
          disabled={isMutating}
          onChange={(event) => setTitle(event.target.value)}
        />

        <input
          type="number"
          min="1"
          value={quantity}
          disabled={isMutating}
          onChange={(event) => setQuantity(Number(event.target.value))}
        />

        <select
          value={priority}
          disabled={isMutating}
          onChange={(event) =>
            setPriority(event.target.value as ShoppingPriority)
          }
        >
          <option value="default">Default</option>
          <option value="low">Low</option>
          <option value="medium">Medium</option>
          <option value="high">High</option>
        </select>

        <input
          type="text"
          value={note}
          placeholder={"Note"}
          disabled={isMutating}
          onChange={(event) => setNote(event.target.value)}
        />

        <button type="submit" disabled={isMutating}>
          Save quantity
        </button>

        <button type="button" disabled={isMutating} onClick={handleDelete}>
          Delete
        </button>

        {error !== null && <p>{error}</p>}
      </form>
    </li>
  );
}
