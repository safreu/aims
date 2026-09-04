import { createContext, useContext } from "react";

export type HouseholdEventType =
  | "shopping_list_changed"
  | "inventory_items_changed"
  | "inventory_categories_changed"
  | "household_resync_required";

type HouseholdEventListener = () => void;

type HouseholdEventsContextValue = {
  subscribe: (
    event: HouseholdEventType,
    listener: HouseholdEventListener,
  ) => () => void;
};

export const HouseholdEventsContext =
  createContext<HouseholdEventsContextValue | null>(null);

export function useHouseholdEvents() {
  const context = useContext(HouseholdEventsContext);

  if (context === null) {
    throw new Error(
      "useHouseholdEvents must be used within a HouseholdEventsProvider",
    );
  }

  return context;
}
