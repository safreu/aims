import { useCallback, useEffect, useState, type ReactNode } from "react";
import { CategoryContext } from "./CategoryContex";
import { useToast } from "../../../../components/toast/ToastContext";
import { useHouseholdEvents } from "../../../households/events/HouseholdEventsContext";
import type { InventoryItemCategory } from "../../types";
import { getInventoryCategories } from "../../api";
import { isHouseholdAccessError } from "../../../households/errors";

type Props = {
  householdId: string;
  children: ReactNode;
};

export function CategoryProvider({ householdId, children }: Props) {
  const { showToast } = useToast();
  const { subscribe } = useHouseholdEvents();

  const [categories, setCategories] = useState<InventoryItemCategory[]>([]);

  const refreshCategories = useCallback(async () => {
    await getInventoryCategories(householdId)
      .then((categories) => setCategories(categories))
      .catch((error) => {
        if (isHouseholdAccessError(error)) return;
        showToast("Failed to fetch categories", "error");
      });
  }, [householdId, showToast]);

  useEffect(() => {
    void refreshCategories();
  }, [refreshCategories]);

  useEffect(() => {
    const unsubscribeCategories = subscribe(
      "inventory_categories_changed",
      () => void refreshCategories(),
    );

    const unsubscribeResync = subscribe(
      "household_resync_required",
      () => void refreshCategories(),
    );

    return () => {
      unsubscribeCategories();
      unsubscribeResync();
    };
  }, [subscribe, refreshCategories]);

  return (
    <CategoryContext.Provider value={{ categories, refreshCategories }}>
      {children}
    </CategoryContext.Provider>
  );
}
