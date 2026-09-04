import { createContext, useContext } from "react";
import type { InventoryItemCategory } from "../../types";

type CategoryContextValue = {
  categories: InventoryItemCategory[];
  refreshCategories: () => Promise<void>;
};

export const CategoryContext = createContext<CategoryContextValue | null>(null);

export function useCategories() {
  const context = useContext(CategoryContext);

  if (context === null) {
    throw new Error("useCategories must be used within a CategoryProvider");
  }

  return context;
}
