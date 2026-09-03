import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import type { InventoryItem } from "../../features/inventory/types";
import { getInventoryItems } from "../../features/inventory/api";
import { InventoryItemRow } from "../../features/inventory/components/rows/InventoryItemRow";
import { ArchivedInventoryItemRow } from "../../features/inventory/components/rows/ArchivedInventoryItemRow";
import "./InventoryPage.css";
import { CreateInventoryItemDialog } from "../../features/inventory/components/dialogs/CreateInventoryItemDialog";
export function InventoryPage() {
  const { householdId } = useParams();

  if (householdId === undefined) {
    throw new Error("InventoryPage requires a householdId");
  }

  const resolvedHouseholdId = householdId;

  const [items, setItems] = useState<InventoryItem[]>([]);
  const [archivedItems, setArchivedItems] = useState<InventoryItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [showCreateItemDialog, setShowCreateItemDialog] = useState(false);

  async function refreshInventory() {
    const [items, archivedItems] = await Promise.all([
      getInventoryItems(resolvedHouseholdId),
      getInventoryItems(resolvedHouseholdId, "archived"),
    ]);

    setItems(items);
    setArchivedItems(archivedItems);
  }

  useEffect(() => {
    void Promise.all([
      getInventoryItems(resolvedHouseholdId),
      getInventoryItems(resolvedHouseholdId, "archived"),
    ])
      .then(([items, archivedItems]) => {
        setItems(items);
        setArchivedItems(archivedItems);
      })
      .catch(() => setError("Failed to load inventory"))
      .finally(() => setLoading(false));
  }, [resolvedHouseholdId]);

  if (loading) {
    return <p>Loading Inventory...</p>;
  }

  if (error !== null) {
    return <p>{error}</p>;
  }

  return (
    <div className="inventory-page">
      <header className="inventory-page__header">
        <div>
          <h1>Inventory</h1>
          <p>Manage your inventory items and stock</p>
        </div>

        <div className="inventory-page__actions">
          <button
            type="button"
            className="button button--primary"
            onClick={() => setShowCreateItemDialog(true)}
          >
            Add item
          </button>
        </div>
      </header>

      <section className="inventory-page__section">
        <div className="inventory-page__section-header">
          <h2>Items</h2>
        </div>

        {items.length === 0 ? (
          <p className="inventory-page__empty">No inventory items :(</p>
        ) : (
          <div className="inventory-list">
            {items.map((item) => (
              <InventoryItemRow
                key={item.id}
                householdId={resolvedHouseholdId}
                item={item}
                onChanged={refreshInventory}
              />
            ))}
          </div>
        )}
      </section>

      <section className="inventory-page__section">
        <div className="inventory-page__section-header">
          <h2>Archived</h2>
        </div>

        {archivedItems.length === 0 ? (
          <p className="inventory-page__empty">No archived items :(</p>
        ) : (
          <div className="inventory-list">
            {archivedItems.map((item) => (
              <ArchivedInventoryItemRow
                key={item.id}
                householdId={resolvedHouseholdId}
                item={item}
                onChanged={refreshInventory}
              />
            ))}
          </div>
        )}
      </section>

      {showCreateItemDialog && (
        <CreateInventoryItemDialog
          householdId={resolvedHouseholdId}
          onCreated={refreshInventory}
          onClose={() => setShowCreateItemDialog(false)}
        />
      )}
    </div>
  );
}
