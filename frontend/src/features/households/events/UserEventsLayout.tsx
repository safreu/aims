import { Outlet } from "react-router-dom";
import { UserEventsProvider } from "./UserEventsProvider";

export function UserEventsLayout() {
  return (
    <UserEventsProvider>
      <Outlet />
    </UserEventsProvider>
  );
}
