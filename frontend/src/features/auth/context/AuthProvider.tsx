import { useCallback, useEffect, useState, type ReactNode } from "react";

import { ApiError, setUnauthorizedHandler } from "../../../api/client";
import { getCurrentUser, logout as logoutRequest } from "../api";
import type { CurrentUser } from "../types";
import { AuthContext } from "./AuthContext";

type AuthProviderProps = {
  children: ReactNode;
};

export function AuthProvider({ children }: AuthProviderProps) {
  const [user, setUser] = useState<CurrentUser | null>(null);

  const [isInitializing, setIsInitializing] = useState(true);

  const [initializationError, setInitializationError] = useState(false);

  const loadCurrentUser = useCallback(async () => {
    try {
      const currentUser = await getCurrentUser();

      setUser(currentUser);
      setInitializationError(false);
    } catch (error) {
      if (error instanceof ApiError && error.status === 401) {
        setUser(null);
        setInitializationError(false);
        return;
      }

      setInitializationError(true);
    } finally {
      setIsInitializing(false);
    }
  }, []);

  async function retryInitialization() {
    setIsInitializing(true);
    setInitializationError(false);

    await loadCurrentUser();
  }

  async function refreshUser() {
    try {
      const currentUser = await getCurrentUser();

      setUser(currentUser);
    } catch (error) {
      if (error instanceof ApiError && error.status === 401) {
        setUser(null);
        return;
      }

      throw error;
    }
  }

  async function logout() {
    await logoutRequest();
    setUser(null);
  }

  useEffect(() => {
    setUnauthorizedHandler(() => {
      setUser(null);
    });

    const timeoutId = window.setTimeout(() => {
      void loadCurrentUser();
    }, 0);

    return () => {
      window.clearTimeout(timeoutId);
      setUnauthorizedHandler(null);
    };
  }, [loadCurrentUser]);

  return (
    <AuthContext.Provider
      value={{
        user,
        isInitializing,
        initializationError,
        retryInitialization,
        refreshUser,
        logout,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}
