import type { PriorityFilterValue } from "../../../../components/list-controls/filters/PriorityFilter";
import type { ShoppingList } from "../../types";

type ShoppingFilters = {
  search: string;
  category: string;
  priority: PriorityFilterValue;
};
export function filterShoppingEntries(
  items: ShoppingList,
  filters: ShoppingFilters,
): ShoppingList {
  const normalizedSearch = filters.search.trim().toLowerCase();

  const inventory_entries = items.inventory_entries.filter((item) => {
    const matchesSearch =
      normalizedSearch === "" ||
      item.name.toLowerCase().includes(normalizedSearch);

    const matchesCategory =
      filters.category === "all" ||
      (filters.category === "uncategorized"
        ? item.category === null
        : item.category?.id === filters.category);

    const matchesPriority =
      filters.priority === "all" || item.priority === filters.priority;

    return matchesSearch && matchesCategory && matchesPriority;
  });

  const custom_entries = items.custom_entries.filter((item) => {
    const matchesSearch =
      normalizedSearch === "" ||
      item.title.toLowerCase().includes(normalizedSearch);

    const matchesPriority =
      filters.priority === "all" || item.priority === filters.priority;

    return matchesSearch && matchesPriority;
  });

  return {
    inventory_entries,
    custom_entries,
  };
}
