import type { CurrentUser } from "../types";
import { useCallback, useEffect, useState, type ReactNode } from "react";
import { getCurrentUser, logout as logoutRequest } from "../api";
import { ApiError, setUnauthorizedHandler } from "../../../api/client";
import { AuthContext } from "./AuthContext";

type AuthProviderProps = {
  children: ReactNode;
};

export function AuthProvider({ children }: AuthProviderProps) {
  const [user, setUser] = useState<CurrentUser | null>(null);
  const [loading, setLoading] = useState(true);
  const [initializationError, setInitializationError] = useState(false);

  const loadCurrentUser = useCallback(async () => {
    return getCurrentUser()
      .then((currentUser) => {
        setUser(currentUser);
        setInitializationError(false);
      })
      .catch((error) => {
        if (error instanceof ApiError && error.status === 401) {
          setUser(null);
          setInitializationError(false);
          return;
        }

        setInitializationError(true);
      })
      .finally(() => setLoading(false));
  }, []);

  async function retryInitialization() {
    setLoading(true);
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

    void loadCurrentUser();

    return () => {
      setUnauthorizedHandler(null);
    };
  }, [loadCurrentUser]);

  return (
    <AuthContext.Provider
      value={{
        user,
        loading,
        initializationError: initializationError,
        retryInitialization,
        refreshUser,
        logout,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}
