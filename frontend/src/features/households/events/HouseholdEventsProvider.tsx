import { useCallback, useEffect, useMemo, useRef, type ReactNode } from "react";
import {
  HouseholdEventsContext,
  type HouseholdEventType,
} from "./HouseholdEventsContext";
import { createHouseholdEventSource } from "../events";

type Props = {
  householdId: string;
  children: ReactNode;
};

type Listener = () => void;

export function HouseholdEventsProvider({ householdId, children }: Props) {
  const listeners = useRef(new Map<HouseholdEventType, Set<Listener>>());

  const subscribe = useCallback(
    (event: HouseholdEventType, listener: Listener) => {
      let eventListeners = listeners.current.get(event);

      if (eventListeners === undefined) {
        eventListeners = new Set();
        listeners.current.set(event, eventListeners);
      }

      eventListeners.add(listener);

      return () => {
        eventListeners.delete(listener);
      };
    },
    [],
  );

  useEffect(() => {
    const eventSource = createHouseholdEventSource(householdId);

    function dispatch(event: HouseholdEventType) {
      listeners.current.get(event)?.forEach((listener) => listener());
    }

    const eventTypes: HouseholdEventType[] = [
      "shopping_list_changed",
      "inventory_items_changed",
      "inventory_categories_changed",
      "household_resync_required",
    ];

    for (const eventType of eventTypes) {
      eventSource.addEventListener(eventType, () => dispatch(eventType));
    }

    return () => eventSource.close();
  }, [householdId]);

  const value = useMemo(() => ({ subscribe }), [subscribe]);

  return (
    <HouseholdEventsContext.Provider value={value}>
      {children}
    </HouseholdEventsContext.Provider>
  );
}
