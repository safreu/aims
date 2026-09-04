import type { CurrentUser } from "../types";
import { useEffect, useState, type ReactNode } from "react";
import { getCurrentUser, logout as logoutRequest } from "../api";
import { ApiError, setUnauthorizedHandler } from "../../../api/client";
import { AuthContext } from "./AuthContext";

type AuthProviderProps = {
  children: ReactNode;
};

export function AuthProvider({ children }: AuthProviderProps) {
  const [user, setUser] = useState<CurrentUser | null>(null);
  const [loading, setLoading] = useState(true);

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

    async function loadCurrentUser() {
      try {
        const currentUser = await getCurrentUser();
        setUser(currentUser);
      } catch (error) {
        if (error instanceof ApiError && error.status === 401) {
          setUser(null);
          return;
        }
        throw error;
      } finally {
        setLoading(false);
      }
    }

    void loadCurrentUser();

    return () => {
      setUnauthorizedHandler(null);
    };
  }, []);

  return (
    <AuthContext.Provider
      value={{
        user,
        loading,
        refreshUser,
        logout,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}
