import { useCallback, useEffect, type ReactNode } from "react";
import { CategoryContext } from "./CategoryContex";
import { useHouseholdEvents } from "../../../households/events/HouseholdEventsContext";
import { getInventoryCategories } from "../../api";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { queryKeys } from "../../../../api/queryKeys";

type Props = {
  householdId: string;
  children: ReactNode;
};

export function CategoryProvider({ householdId, children }: Props) {
  const { subscribe } = useHouseholdEvents();

  const queryClient = useQueryClient();

  const { data: categories = [] } = useQuery({
    queryKey: queryKeys.categories(householdId),
    queryFn: () => getInventoryCategories(householdId),
  });

  const refreshCategories = useCallback(() => {
    return queryClient.invalidateQueries({
      queryKey: queryKeys.categories(householdId),
    });
  }, [queryClient, householdId]);

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
