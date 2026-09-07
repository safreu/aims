import { useCallback, useEffect, useMemo, useRef, type ReactNode } from "react";
import { createUserEventSource } from "../events";
import { UserEventsContext, type UserEventType } from "./UserEventsContext";

type Props = {
  children: ReactNode;
};

type Listener = () => void;

export function UserEventsProvider({ children }: Props) {
  const listeners = useRef(new Map<UserEventType, Set<Listener>>());

  const subscribe = useCallback((event: UserEventType, listener: Listener) => {
    let eventListeners = listeners.current.get(event);

    if (eventListeners === undefined) {
      eventListeners = new Set();
      listeners.current.set(event, eventListeners);
    }

    eventListeners.add(listener);

    return () => {
      eventListeners.delete(listener);
    };
  }, []);

  useEffect(() => {
    const eventSource = createUserEventSource();

    function dispatch(event: UserEventType) {
      listeners.current.get(event)?.forEach((listener) => listener());
    }

    const eventTypes: UserEventType[] = ["household_memberships_changed"];

    for (const eventType of eventTypes) {
      eventSource.addEventListener(eventType, () => dispatch(eventType));
    }

    return () => eventSource.close();
  }, []);

  const value = useMemo(() => ({ subscribe }), [subscribe]);

  return (
    <UserEventsContext.Provider value={value}>
      {children}
    </UserEventsContext.Provider>
  );
}
