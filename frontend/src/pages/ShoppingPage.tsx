import { useEffect, useState, type SubmitEvent } from "react";
import { useParams } from "react-router-dom";
import type {
  ShoppingList,
  ShoppingPriority,
} from "../features/shopping/types";
import {
  createCustomShoppingEntry,
  getShoppingList,
} from "../features/shopping/api";
import { InventoryShoppingEntryRow } from "../features/shopping/InventoryShoppingEntryRow";
import { CustomShoppingEntryRow } from "../features/shopping/CustomShoppingEntryRow";
import { subscribeToHouseholdEvents } from "../features/households/events";

export function ShoppingPage() {
  const { householdId } = useParams();

  if (householdId === undefined) {
    throw new Error("ShoppingPage requires a householdId");
  }

  const resolvedHouseholdId = householdId;

  const [shoppingList, setShoppingList] = useState<ShoppingList | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [title, setTitle] = useState("");
  const [quantity, setQuantity] = useState(1);
  const [priority, setPriority] = useState<ShoppingPriority>("default");
  const [note, setNote] = useState("");

  const [createError, setCreateError] = useState<string | null>(null);
  const [isCreating, setIsCreating] = useState(false);

  async function refreshShoppingList() {
    const shoppingList = await getShoppingList(resolvedHouseholdId);
    setShoppingList(shoppingList);
  }

  function handleCreate(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    const trimmedTitle = title.trim();

    if (trimmedTitle === "") {
      setCreateError("Title is required");
      return;
    }

    setIsCreating(true);
    setCreateError(null);

    void createCustomShoppingEntry(resolvedHouseholdId, {
      title: trimmedTitle,
      quantity,
      priority,
      note: note.trim() === "" ? null : note.trim(),
    })
      .then(() => {
        setTitle("");
        setQuantity(1);
        setPriority("default");
        setNote("");

        return refreshShoppingList();
      })
      .catch(() => setCreateError("Failed to create shopping item"))
      .finally(() => setIsCreating(false));
  }

  useEffect(() => {
    void getShoppingList(resolvedHouseholdId)
      .then((shoppingList) => setShoppingList(shoppingList))
      .catch(() => setError("Failed to load shopping list"))
      .finally(() => setLoading(false));
  }, [resolvedHouseholdId]);

  useEffect(() => {
    const eventSource = subscribeToHouseholdEvents(resolvedHouseholdId, () => {
      void refreshShoppingList();
    });
    return () => eventSource.close();
  }, [resolvedHouseholdId]);

  if (loading) {
    return <p>Loading shopping list...</p>;
  }

  if (error !== null) {
    return <p>{error}</p>;
  }

  if (shoppingList === null) {
    return null;
  }

  return (
    <main>
      <h1>Shopping</h1>

      <form onSubmit={handleCreate}>
        <h2>Add item</h2>

        <input
          type="text"
          placeholder="item"
          value={title}
          disabled={isCreating}
          onChange={(event) => setTitle(event.target.value)}
          required
        />

        <input
          type="number"
          min="0"
          value={quantity}
          disabled={isCreating}
          onChange={(event) => setQuantity(Number(event.target.value))}
        />

        <select
          value={priority}
          disabled={isCreating}
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
          placeholder="note"
          value={note}
          disabled={isCreating}
          onChange={(event) => setNote(event.target.value)}
        />

        <button type="submit" disabled={isCreating}>
          Add
        </button>

        {createError !== null && <p>{createError}</p>}
      </form>

      <h2>Inventory items</h2>

      {shoppingList.inventory_entries.length === 0 ? (
        <p>No inventory items need to be bought :)</p>
      ) : (
        <ul>
          {shoppingList.inventory_entries.map((entry) => (
            <InventoryShoppingEntryRow
              key={entry.item_id}
              householdId={resolvedHouseholdId}
              entry={entry}
              onChange={refreshShoppingList}
            />
          ))}
        </ul>
      )}

      <h2>custom items</h2>

      {shoppingList.custom_entries.length === 0 ? (
        <p>No custom items need to be bought :)</p>
      ) : (
        <ul>
          {shoppingList.custom_entries.map((entry) => (
            <CustomShoppingEntryRow
              key={entry.id}
              householdId={resolvedHouseholdId}
              entry={entry}
              onChange={refreshShoppingList}
            />
          ))}
        </ul>
      )}
    </main>
  );
}
