import { useCallback, useEffect, useState } from "react";
import { useParams } from "react-router-dom";
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
import { queryKeys } from "../../api/queryKeys";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import Skeleton from "react-loading-skeleton";
export function InventoryPage() {
  const { householdId } = useParams();
  const { subscribe } = useHouseholdEvents();

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

  const queryClient = useQueryClient();

  const refreshInventory = useCallback(() => {
    return queryClient.invalidateQueries({
      queryKey: queryKeys.inventory.all(resolvedHouseholdId),
    });
  }, [resolvedHouseholdId, queryClient]);

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

        {isItemsPending ? (
          <InventoryItemsSkeleton />
        ) : isItemsError ? (
          <p className="inventory-page__empty">
            Failed to load inventory items
          </p>
        ) : visibleItems.length === 0 ? (
          <p className="inventory-page__empty">
            {items.length === 0
              ? "No inventory items :("
              : "No items match your filtering"}
          </p>
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

        {isArchivedItemsPending ? (
          <ArchivedInventoryItemsSkeleton />
        ) : isArchivedItemsError ? (
          <p className="inventory-page__empty">Failed to load archived items</p>
        ) : archivedItems.length === 0 ? (
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

function InventoryItemsSkeleton() {
  return (
    <div className="inventory-list" aria-label="Loading inventory items">
      {Array.from({ length: 3 }).map((_, index) => (
        <div
          key={index}
          className="inventory-item-row inventory-item-row--skeleton"
        >
          <div className="inventory-item-row__info">
            <strong>
              <Skeleton width="9rem" />
            </strong>

            <span>
              <Skeleton width="5rem" />
            </span>
          </div>

          <div className="inventory-item-row__stock">
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
    <div className="inventory-list" aria-label="Loading archived items">
      {Array.from({ length: 2 }).map((_, index) => (
        <div key={index} className="archived-inventory-item-row">
          <div className="archived-inventory-item-row__info">
            <strong>
              <Skeleton width="8rem" />
            </strong>

            <span>
              <Skeleton width="5rem" />
            </span>
          </div>

          <Skeleton width={75} height={36} borderRadius="var(--radius-sm)" />
        </div>
      ))}
    </div>
  );
}
