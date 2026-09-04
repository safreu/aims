import type { PriorityFilterValue } from "../../../../components/list-controls/filters/PriorityFilter";
import type { InventoryItem } from "../../types";

type InventoryFilters = {
  search: string;
  category: string;
  priority: PriorityFilterValue;
};

export function filterInventoryItems(
  items: InventoryItem[],
  filters: InventoryFilters,
): InventoryItem[] {
  const normalizedSearch = filters.search.trim().toLowerCase();

  return items.filter((item) => {
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
}
