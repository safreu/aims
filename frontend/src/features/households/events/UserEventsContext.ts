import { createContext, useContext } from "react";

export type UserEventType = "household_memberships_changed";

type UserEventListener = () => void;

type UserEventsContextValue = {
  subscribe: (event: UserEventType, listener: UserEventListener) => () => void;
};

export const UserEventsContext = createContext<UserEventsContextValue | null>(
  null,
);

export function useUserEvents() {
  const context = useContext(UserEventsContext);

  if (context === null) {
    throw new Error("useUserEvents must be used within a UserEventsProvider");
  }

  return context;
}
