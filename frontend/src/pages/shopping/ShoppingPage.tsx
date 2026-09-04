import { useCallback, useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import type { ShoppingList } from "../../features/shopping/types";
import { getShoppingList } from "../../features/shopping/api";
import { InventoryShoppingEntryRow } from "../../features/shopping/components/rows/InventoryShoppingEntryRow";
import { CustomShoppingEntryRow } from "../../features/shopping/components/rows/CustomShoppingEntryRow";
import { CreateShoppingEntryDialog } from "../../features/shopping/components/dialogs/CreateShoppingEntryDialog";
import "./ShoppingPage.css";
import { AddItemAction } from "../../components/actions/AddItemAction";
import { ListControls } from "../../components/list-controls/ListControls";
import { filterShoppingEntries } from "../../features/shopping/components/list/filterShoppingEntries";
import { useHouseholdEvents } from "../../features/households/events/HouseholdEventsContext";
import {
  CategoryFilter,
  type CategoryFilterValue,
} from "../../components/list-controls/filters/CategoryFilter";
import {
  PriorityFilter,
  type PriorityFilterValue,
} from "../../components/list-controls/filters/PriorityFilter";
import { orderShoppingEntries } from "../../features/shopping/components/list/orderShoppingEntries";
import {
  OrderBy,
  type OrderByValue,
} from "../../components/list-controls/OderBy";

export function ShoppingPage() {
  const { householdId } = useParams();
  const { subscribe } = useHouseholdEvents();

  if (householdId === undefined) {
    throw new Error("ShoppingPage requires a householdId");
  }

  const resolvedHouseholdId = householdId;

  const [shoppingList, setShoppingList] = useState<ShoppingList | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [showCreateDialog, setShowCreateDialog] = useState(false);

  const [search, setSearch] = useState("");
  const [categoryFilter, setCategoryFilter] =
    useState<CategoryFilterValue>("all");
  const [priorityFilter, setPriorityFilter] =
    useState<PriorityFilterValue>("all");
  const [orderBy, setOrderBy] = useState<OrderByValue>("default");

  const canReset =
    categoryFilter !== "all" ||
    priorityFilter !== "all" ||
    orderBy !== "default";

  function resetListControls() {
    setCategoryFilter("all");
    setPriorityFilter("all");
    setOrderBy("default");
  }

  const visibleItems = filterShoppingEntries(
    shoppingList ?? { inventory_entries: [], custom_entries: [] },
    {
      search,
      category: categoryFilter,
      priority: priorityFilter,
    },
  );

  const orderedItems = orderShoppingEntries(visibleItems, orderBy);

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
    const unsubscribeCategories = subscribe(
      "inventory_categories_changed",
      () => void refreshShoppingList(),
    );

    const unsubscribeResync = subscribe(
      "household_resync_required",
      () => void refreshShoppingList(),
    );

    return () => {
      unsubscribeCategories();
      unsubscribeResync();
    };
  }, [subscribe, refreshShoppingList]);

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

  const activeFilterCount =
    Number(categoryFilter !== "all") + Number(priorityFilter !== "all");

  return (
    <main className="shopping-page">
      <header className="shopping-page__header">
        <div>
          <h1>Shopping</h1>
          <p>Your household shopping list</p>
        </div>

        <AddItemAction onClick={() => setShowCreateDialog(true)} />
      </header>

      <div>
        <ListControls
          search={search}
          onSearchChange={setSearch}
          searchPlaceholder="Search inventory..."
          activeFilterCount={activeFilterCount}
          canReset={canReset}
          onReset={resetListControls}
        >
          <CategoryFilter
            value={categoryFilter}
            onValueChange={setCategoryFilter}
          />

          <PriorityFilter
            value={priorityFilter}
            onValueChange={setPriorityFilter}
          />

          <OrderBy value={orderBy} onValueChange={setOrderBy} />
        </ListControls>
      </div>

      <section className="shopping-page__section">
        <div className="shopping-page__section-header">
          <h2>Inventory items</h2>

          <span className="shopping-page__count">
            {remainingInventoryCount}
          </span>
        </div>

        {orderedItems.inventory_entries.length === 0 ? (
          <p className="shopping-page__empty">
            No inventory items need to be bought :)
          </p>
        ) : (
          <ul className="shopping-list">
            {orderedItems.inventory_entries.map((entry) => (
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

        {orderedItems.custom_entries.length === 0 ? (
          <p className="shopping-page__empty">
            No custom items need to be bought :)
          </p>
        ) : (
          <ul className="shopping-list">
            {orderedItems.custom_entries.map((entry) => (
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
