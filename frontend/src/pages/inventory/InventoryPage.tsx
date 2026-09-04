import { useCallback, useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import type { InventoryItem } from "../../features/inventory/types";
import { getInventoryItems } from "../../features/inventory/api";
import { InventoryItemRow } from "../../features/inventory/components/rows/InventoryItemRow";
import { ArchivedInventoryItemRow } from "../../features/inventory/components/rows/ArchivedInventoryItemRow";
import "./InventoryPage.css";
import { CreateInventoryItemDialog } from "../../features/inventory/components/dialogs/CreateInventoryItemDialog";
import { AddItemAction } from "../../components/actions/AddItemAction";
import { ListControls } from "../../components/list-controls/ListControls";
import { filterInventoryItems } from "../../features/inventory/components/search/filterInventoryItems";
import {
  CategoryFilter,
  type CategoryFilterValue,
} from "../../components/list-controls/filters/CategoryFilter";
import { useHouseholdEvents } from "../../features/households/events/HouseholdEventsContext";
import {
  PriorityFilter,
  type PriorityFilterValue,
} from "../../components/list-controls/filters/PriorityFilter";
export function InventoryPage() {
  const { householdId } = useParams();
  const { subscribe } = useHouseholdEvents();

  if (householdId === undefined) {
    throw new Error("InventoryPage requires a householdId");
  }

  const resolvedHouseholdId = householdId;

  const [items, setItems] = useState<InventoryItem[]>([]);
  const [archivedItems, setArchivedItems] = useState<InventoryItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [showCreateItemDialog, setShowCreateItemDialog] = useState(false);

  const [search, setSearch] = useState("");
  const [categoryFilter, setCategoryFilter] =
    useState<CategoryFilterValue>("all");
  const [priorityFilter, setPriorityFilter] =
    useState<PriorityFilterValue>("all");

  const visibleItems = filterInventoryItems(items, {
    search,
    category: categoryFilter,
    priority: priorityFilter,
  });

  const refreshInventory = useCallback(async () => {
    const [items, archivedItems] = await Promise.all([
      getInventoryItems(resolvedHouseholdId),
      getInventoryItems(resolvedHouseholdId, "archived"),
    ]);

    setItems(items);
    setArchivedItems(archivedItems);
  }, [resolvedHouseholdId]);

  useEffect(() => {
    const unsubscribeCategories = subscribe(
      "inventory_categories_changed",
      () => void refreshInventory(),
    );

    const unsubscribeInventory = subscribe(
      "inventory_items_changed",
      () => void refreshInventory(),
    );

    const unsubscribeResync = subscribe(
      "household_resync_required",
      () => void refreshInventory(),
    );

    return () => {
      unsubscribeCategories();
      unsubscribeInventory();
      unsubscribeResync();
    };
  }, [subscribe, refreshInventory]);

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

  const activeFilterCount =
    Number(categoryFilter !== "all") + Number(priorityFilter !== "all");

  return (
    <div className="inventory-page">
      <header className="inventory-page__header">
        <div>
          <h1>Inventory</h1>
          <p>Manage your inventory items and stock</p>
        </div>

        <div className="inventory-page__actions">
          <AddItemAction onClick={() => setShowCreateItemDialog(true)} />
        </div>
      </header>

      <div>
        <ListControls
          search={search}
          onSearchChange={setSearch}
          searchPlaceholder="Search inventory..."
          activeFilterCount={activeFilterCount}
        >
          <CategoryFilter
            value={categoryFilter}
            onValueChange={setCategoryFilter}
          />

          <PriorityFilter
            value={priorityFilter}
            onValueChange={setPriorityFilter}
          />
        </ListControls>
      </div>

      <section className="inventory-page__section">
        <div className="inventory-page__section-header">
          <h2>Items</h2>
        </div>

        {items.length === 0 ? (
          <p className="inventory-page__empty">No inventory items :(</p>
        ) : (
          <div className="inventory-list">
            {visibleItems.map((item) => (
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
