import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useState } from "react";
import Skeleton from "react-loading-skeleton";
import { useParams } from "react-router-dom";

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
import { useHouseholdEvents } from "../../households/events/HouseholdEventsContext";
import { getInventoryItems } from "../api";
import { CreateInventoryItemDialog } from "../components/dialogs/CreateInventoryItemDialog";
import { ArchivedInventoryItemRow } from "../components/rows/ArchivedInventoryItemRow";
import { InventoryItemRow } from "../components/rows/InventoryItemRow";
import { filterInventoryItems } from "../components/search/filterInventoryItems";

import styles from "./InventoryPage.module.css";

export function InventoryPage() {
  const { householdId } = useParams();
  const { subscribe } = useHouseholdEvents();
  const queryClient = useQueryClient();

  if (householdId === undefined) {
    throw new Error("InventoryPage requires a householdId");
  }

  const resolvedHouseholdId = householdId;

  const {
    data: items = [],
    isPending: isItemsPending,
    isError: isItemsError,
  } = useQuery({
    queryKey: queryKeys.inventory.active(resolvedHouseholdId),
    queryFn: () => getInventoryItems(resolvedHouseholdId),
  });

  const {
    data: archivedItems = [],
    isPending: isArchivedItemsPending,
    isError: isArchivedItemsError,
  } = useQuery({
    queryKey: queryKeys.inventory.archived(resolvedHouseholdId),
    queryFn: () => getInventoryItems(resolvedHouseholdId, "archived"),
  });

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

  const activeFilterCount =
    Number(categoryFilter !== "all") + Number(priorityFilter !== "all");

  const refreshInventory = useCallback(() => {
    return queryClient.invalidateQueries({
      queryKey: queryKeys.inventory.all(resolvedHouseholdId),
    });
  }, [queryClient, resolvedHouseholdId]);

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

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <div>
          <h1>Inventory</h1>
          <p>Manage your inventory items and stock</p>
        </div>

        <div className={styles.actions}>
          <AddItemAction onClick={() => setShowCreateItemDialog(true)} />
        </div>
      </header>

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

      <section className={styles.section}>
        <div className={styles.sectionHeader}>
          <h2>Items</h2>
        </div>

        {isItemsPending ? (
          <InventoryItemsSkeleton />
        ) : isItemsError ? (
          <p className={styles.message} role="alert">
            Failed to load inventory items
          </p>
        ) : visibleItems.length === 0 ? (
          <p className={styles.message}>
            {items.length === 0
              ? "No inventory items yet"
              : "No items match your filters"}
          </p>
        ) : (
          <div className={styles.list}>
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

      <section className={styles.section}>
        <div className={styles.sectionHeader}>
          <h2>Archived</h2>
        </div>

        {isArchivedItemsPending ? (
          <ArchivedInventoryItemsSkeleton />
        ) : isArchivedItemsError ? (
          <p className={styles.message} role="alert">
            Failed to load archived items
          </p>
        ) : archivedItems.length === 0 ? (
          <p className={styles.message}>No archived items</p>
        ) : (
          <div className={styles.list}>
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

function InventoryItemsSkeleton() {
  return (
    <div className={styles.list} aria-label="Loading inventory items">
      {Array.from({ length: 3 }).map((_, index) => (
        <div key={index} className={styles.itemSkeleton}>
          <div className={styles.itemSkeletonInfo}>
            <Skeleton width="9rem" />
            <Skeleton width="5rem" />
          </div>

          <div className={styles.itemSkeletonStock}>
            <Skeleton width="2.5rem" height="0.75rem" />
            <Skeleton width="1.5rem" height="1.35rem" />
          </div>
        </div>
      ))}
    </div>
  );
}

function ArchivedInventoryItemsSkeleton() {
  return (
    <div className={styles.list} aria-label="Loading archived items">
      {Array.from({ length: 2 }).map((_, index) => (
        <div key={index} className={styles.archivedSkeleton}>
          <div className={styles.archivedSkeletonInfo}>
            <Skeleton width="8rem" />
            <Skeleton width="5rem" />
          </div>

          <Skeleton width={75} height={36} borderRadius="var(--radius-sm)" />
        </div>
      ))}
    </div>
  );
}
