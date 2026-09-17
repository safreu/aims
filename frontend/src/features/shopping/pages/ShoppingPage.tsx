import { useCallback, useEffect, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useParams } from "react-router-dom";
import Skeleton from "react-loading-skeleton";

import { queryKeys } from "../../../api/queryKeys";
import { AddItemAction } from "../../../components/actions/AddItemAction";
import {
  CategoryFilter,
  type CategoryFilterValue,
} from "../../../components/list-controls/filters/CategoryFilter";
import {
  PriorityFilter,
  type PriorityFilterValue,
} from "../../../components/list-controls/filters/PriorityFilter";
import { ListControls } from "../../../components/list-controls/ListControls";
import {
  OrderBy,
  type OrderByValue,
} from "../../../components/list-controls/OrderBy";
import { useHouseholdEvents } from "../../households/events/HouseholdEventsContext";
import { getShoppingList } from "../api";
import { CreateShoppingEntryDialog } from "../components/dialogs/CreateShoppingEntryDialog";
import { filterShoppingEntries } from "../components/list/filterShoppingEntries";
import { orderShoppingEntries } from "../components/list/orderShoppingEntries";
import { CustomShoppingEntryRow } from "../components/rows/CustomShoppingEntryRow";
import { InventoryShoppingEntryRow } from "../components/rows/InventoryShoppingEntryRow";

import rowStyles from "../components/rows/ShoppingEntryRow.module.css";
import styles from "./ShoppingPage.module.css";

export function ShoppingPage() {
  const { householdId } = useParams();
  const { subscribe } = useHouseholdEvents();

  if (householdId === undefined) {
    throw new Error("ShoppingPage requires a householdId");
  }

  const resolvedHouseholdId = householdId;
  const queryClient = useQueryClient();

  const {
    data: shoppingList,
    isPending,
    isError,
  } = useQuery({
    queryKey: queryKeys.shopping(resolvedHouseholdId),
    queryFn: () => getShoppingList(resolvedHouseholdId),
  });

  const refreshShoppingList = useCallback(() => {
    return queryClient.invalidateQueries({
      queryKey: queryKeys.shopping(resolvedHouseholdId),
    });
  }, [queryClient, resolvedHouseholdId]);

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

  const hasSearchOrFilters =
    search.trim() !== "" ||
    categoryFilter !== "all" ||
    priorityFilter !== "all";

  function resetListControls() {
    setCategoryFilter("all");
    setPriorityFilter("all");
    setOrderBy("default");
  }

  const visibleItems = filterShoppingEntries(
    shoppingList ?? {
      inventory_entries: [],
      custom_entries: [],
    },
    {
      search,
      category: categoryFilter,
      priority: priorityFilter,
    },
  );

  const orderedItems = orderShoppingEntries(visibleItems, orderBy);

  useEffect(() => {
    const unsubscribeShopping = subscribe(
      "shopping_list_changed",
      () => void refreshShoppingList(),
    );

    const unsubscribeCategories = subscribe(
      "inventory_categories_changed",
      () => void refreshShoppingList(),
    );

    const unsubscribeResync = subscribe(
      "household_resync_required",
      () => void refreshShoppingList(),
    );

    return () => {
      unsubscribeShopping();
      unsubscribeCategories();
      unsubscribeResync();
    };
  }, [subscribe, refreshShoppingList]);
  const remainingInventoryCount =
    shoppingList?.inventory_entries.filter((entry) => !entry.checked).length ??
    0;

  const remainingCustomCount =
    shoppingList?.custom_entries.filter((entry) => !entry.checked).length ?? 0;

  const activeFilterCount =
    Number(categoryFilter !== "all") + Number(priorityFilter !== "all");

  return (
    <main className={styles.page}>
      <header className={styles.header}>
        <div>
          <h1>Shopping</h1>
          <p>Your household shopping list</p>
        </div>

        <div className={styles.addButton}>
          <AddItemAction onClick={() => setShowCreateDialog(true)} />
        </div>
      </header>

      <ListControls
        search={search}
        onSearchChange={setSearch}
        searchPlaceholder="Search shopping list..."
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

      <section className={styles.section}>
        <div className={styles.sectionHeader}>
          <h2>Inventory items</h2>

          {!isPending && !isError && (
            <span
              className={styles.count}
              aria-label={`${remainingInventoryCount} inventory items remaining`}
            >
              {remainingInventoryCount}
            </span>
          )}
        </div>

        {isPending ? (
          <ShoppingEntriesSkeleton />
        ) : isError ? (
          <p className={styles.empty}>Failed to load shopping list</p>
        ) : orderedItems.inventory_entries.length === 0 ? (
          <p className={styles.empty}>
            {hasSearchOrFilters
              ? "No inventory items match the current search or filters."
              : "No inventory items need to be bought."}
          </p>
        ) : (
          <ul className={rowStyles.list}>
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

      <section className={styles.section}>
        <div className={styles.sectionHeader}>
          <h2>Custom items</h2>

          {!isPending && !isError && (
            <span
              className={styles.count}
              aria-label={`${remainingCustomCount} custom items remaining`}
            >
              {remainingCustomCount}
            </span>
          )}
        </div>

        {isPending ? (
          <ShoppingEntriesSkeleton />
        ) : isError ? (
          <p className={styles.empty}>Failed to load shopping list</p>
        ) : orderedItems.custom_entries.length === 0 ? (
          <p className={styles.empty}>
            {hasSearchOrFilters
              ? "No custom items match the current search or filters."
              : "No custom items need to be bought."}
          </p>
        ) : (
          <ul className={rowStyles.list}>
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

function ShoppingEntriesSkeleton() {
  return (
    <ul className={rowStyles.list} aria-label="Loading shopping list">
      {Array.from({ length: 3 }).map((_, index) => (
        <li key={index} className={`${rowStyles.entry} ${rowStyles.skeleton}`}>
          <Skeleton width={20} height={20} borderRadius="4px" />

          <div className={rowStyles.open}>
            <div className={rowStyles.main}>
              <div className={rowStyles.title}>
                <Skeleton width="9rem" height="0.9rem" />
              </div>

              <Skeleton width="2rem" height="0.9rem" />
            </div>

            <div className={rowStyles.meta}>
              <Skeleton width="5rem" height="0.8rem" />
            </div>
          </div>
        </li>
      ))}
    </ul>
  );
}
