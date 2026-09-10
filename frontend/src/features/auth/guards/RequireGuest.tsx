import { AuthGuard } from "./AuthGuard";

export function RequireGuest() {
  return <AuthGuard mode="guest" />;
}
