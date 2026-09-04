import { useCallback, useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import type { ShoppingList } from "../../features/shopping/types";
import { getShoppingList } from "../../features/shopping/api";
import { InventoryShoppingEntryRow } from "../../features/shopping/components/rows/InventoryShoppingEntryRow";
import { CustomShoppingEntryRow } from "../../features/shopping/components/rows/CustomShoppingEntryRow";
import { subscribeToHouseholdEvents } from "../../features/households/events";
import { CreateShoppingEntryDialog } from "../../features/shopping/components/dialogs/CreateShoppingEntryDialog";
import "./ShoppingPage.css";

export function ShoppingPage() {
  const { householdId } = useParams();

  if (householdId === undefined) {
    throw new Error("ShoppingPage requires a householdId");
  }

  const resolvedHouseholdId = householdId;

  const [shoppingList, setShoppingList] = useState<ShoppingList | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [showCreateDialog, setShowCreateDialog] = useState(false);

  const refreshShoppingList = useCallback(async () => {
    const shoppingList = await getShoppingList(resolvedHouseholdId);
    setShoppingList(shoppingList);
  }, [resolvedHouseholdId]);

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
  }, [resolvedHouseholdId, refreshShoppingList]);

  if (loading) {
    return <p>Loading shopping list...</p>;
  }

  if (error !== null) {
    return <p>{error}</p>;
  }

  if (shoppingList === null) {
    return null;
  }

  const remainingInventoryCount = shoppingList.inventory_entries.filter(
    (entry) => !entry.checked,
  ).length;

  const remainingCustomCount = shoppingList.custom_entries.filter(
    (entry) => !entry.checked,
  ).length;

  const sortedInventoryEntries = [...shoppingList.inventory_entries].sort(
    (a, b) => Number(a.checked) - Number(b.checked),
  );

  const sortedCustomEntries = [...shoppingList.custom_entries].sort(
    (a, b) => Number(a.checked) - Number(b.checked),
  );

  return (
    <main className="shopping-page">
      <header className="shopping-page__header">
        <div>
          <h1>Shopping</h1>
          <p>Your household shopping list</p>
        </div>

        <button
          type="button"
          className="button button--primary"
          onClick={() => setShowCreateDialog(true)}
        >
          Add item
        </button>
      </header>

      <section className="shopping-page__section">
        <div className="shopping-page__section-header">
          <h2>Inventory items</h2>

          <span className="shopping-page__count">
            {remainingInventoryCount}
          </span>
        </div>

        {shoppingList.inventory_entries.length === 0 ? (
          <p className="shopping-page__empty">
            No inventory items need to be bought :)
          </p>
        ) : (
          <ul className="shopping-list">
            {sortedInventoryEntries.map((entry) => (
              <InventoryShoppingEntryRow
                key={entry.item_id}
                householdId={resolvedHouseholdId}
                entry={entry}
                onChange={refreshShoppingList}
              />
            ))}
          </ul>
        )}
      </section>

      <section className="shopping-page__section">
        <div className="shopping-page__section-header">
          <h2>custom items</h2>
          <span className="shopping-page__count">{remainingCustomCount}</span>
        </div>

        {shoppingList.custom_entries.length === 0 ? (
          <p className="shopping-page__empty">
            No custom items need to be bought :)
          </p>
        ) : (
          <ul className="shopping-list">
            {sortedCustomEntries.map((entry) => (
              <CustomShoppingEntryRow
                key={entry.id}
                householdId={resolvedHouseholdId}
                entry={entry}
                onChange={refreshShoppingList}
              />
            ))}
          </ul>
        )}
      </section>

      {showCreateDialog && (
        <CreateShoppingEntryDialog
          householdId={resolvedHouseholdId}
          onCreated={refreshShoppingList}
          onClose={() => setShowCreateDialog(false)}
        />
      )}
    </main>
  );
}
