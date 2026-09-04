import { Navigate, Outlet } from "react-router-dom";
import { useAuth } from "../context/AuthContext";

export function RequireAuth() {
  const { user, loading } = useAuth();

  if (loading) {
    return <p>loading...</p>;
  }

  if (user === null) {
    return <Navigate to="/login" replace />;
  }

  return <Outlet />;
}
