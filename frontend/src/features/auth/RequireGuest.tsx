import { Navigate, Outlet } from "react-router-dom";
import { useAuth } from "./AuthProvider";

export function RequireGuest() {
  const { user, loading } = useAuth();

  if (loading) {
    return <p>loading...</p>;
  }

  if (user !== null) {
    return <Navigate to="/households" replace />;
  }

  return <Outlet />;
}
