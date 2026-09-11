import { AuthGuard } from "./AuthGuard";

export function RequireAuth() {
  return <AuthGuard mode="authenticated" />;
}
