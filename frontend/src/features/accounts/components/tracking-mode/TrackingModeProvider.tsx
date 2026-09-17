import { useState, type ReactNode } from "react";

import { useAuth } from "../../../auth/context/AuthContext";
import { TrackingModeContext, type TrackingMode } from "./TrackingModeContext";

type Props = {
  children: ReactNode;
};

function loadTrackingMode(storageKey: string): TrackingMode {
  const stored = localStorage.getItem(storageKey);

  if (stored === "manual" || stored === "qr") {
    return stored;
  }

  return "qr";
}

export function TrackingModeProvider({ children }: Props) {
  const { user } = useAuth();

  if (user === null) {
    return children;
  }

  return (
    <AuthenticatedTrackingModeProvider key={user.id} userId={user.id}>
      {children}
    </AuthenticatedTrackingModeProvider>
  );
}

type AuthenticatedTrackingModeProviderProps = {
  userId: string;
  children: ReactNode;
};

function AuthenticatedTrackingModeProvider({
  userId,
  children,
}: AuthenticatedTrackingModeProviderProps) {
  const storageKey = `inventory-tracking-mode:${userId}`;

  const [trackingMode, setTrackingModeState] = useState<TrackingMode>(() =>
    loadTrackingMode(storageKey),
  );

  function setTrackingMode(mode: TrackingMode) {
    localStorage.setItem(storageKey, mode);
    setTrackingModeState(mode);
  }

  return (
    <TrackingModeContext.Provider
      value={{
        trackingMode,
        setTrackingMode,
      }}
    >
      {children}
    </TrackingModeContext.Provider>
  );
}
