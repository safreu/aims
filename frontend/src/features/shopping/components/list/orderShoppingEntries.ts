import type { OrderByValue } from "../../../../components/list-controls/OderBy";
import type { ShoppingList, ShoppingPriority } from "../../types";

const priorityRank = {
  default: 0,
  low: 1,
  medium: 2,
  high: 3,
};

function compareEntries(
  a: { checked: boolean; quantity: number; priority: ShoppingPriority },
  b: { checked: boolean; quantity: number; priority: ShoppingPriority },
  orderBy: OrderByValue,
): number {
  const checkedComparison = Number(a.checked) - Number(b.checked);

  if (checkedComparison !== 0) return checkedComparison;

  switch (orderBy) {
    case "quantity-desc":
      return b.quantity - a.quantity;
    case "quantity-asc":
      return a.quantity - b.quantity;
    case "priority-desc":
      return priorityRank[b.priority] - priorityRank[a.priority];
    case "priority-asc":
      return priorityRank[a.priority] - priorityRank[b.priority];
    case "default":
      return 0;
  }
}

export function orderShoppingEntries(
  items: ShoppingList,
  orderBy: OrderByValue,
): ShoppingList {
  return {
    inventory_entries: [...items.inventory_entries].sort((a, b) =>
      compareEntries(a, b, orderBy),
    ),
    custom_entries: [...items.custom_entries].sort((a, b) =>
      compareEntries(a, b, orderBy),
    ),
  };
}
