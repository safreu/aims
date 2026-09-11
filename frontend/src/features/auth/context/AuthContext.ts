import { createContext, useContext } from "react";
import type { CurrentUser } from "../types";

type AuthContextValue = {
  user: CurrentUser | null;
  loading: boolean;
  initializationError: boolean;
  retryInitialization: () => Promise<void>;
  refreshUser: () => Promise<void>;
  logout: () => Promise<void>;
};

export const AuthContext = createContext<AuthContextValue | null>(null);

export function useAuth(): AuthContextValue {
  const context = useContext(AuthContext);

  if (context === null) {
    throw new Error("useAuth must be used inside AuthProvider");
  }
  return context;
}
