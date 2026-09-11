import { Navigate, Outlet } from "react-router-dom";
import { AuthErrorScreen } from "../components/AuthErrorScreen";
import { AuthLoadingScreen } from "../components/AuthLoadingScreen";
import { useAuth } from "../context/AuthContext";

type Props = { mode: "authenticated" | "guest" };

export function AuthGuard({ mode }: Props) {
  const { user, loading, initializationError, retryInitialization } = useAuth();

  if (loading) {
    return <AuthLoadingScreen />;
  }

  if (initializationError) {
    return <AuthErrorScreen onRetry={() => void retryInitialization()} />;
  }

  if (mode === "authenticated" && user === null) {
    return <Navigate to="/login" replace />;
  }

  if (mode === "guest" && user !== null) {
    return <Navigate to="/households" replace />;
  }

  return <Outlet />;
}
