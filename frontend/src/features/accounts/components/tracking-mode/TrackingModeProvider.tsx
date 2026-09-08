import { useState, type ReactNode } from "react";
import { TrackingModeContext, type TrackingMode } from "./TrackingModeContext";
import { useAuth } from "../../../auth/context/AuthContext";

type Props = {
  children: ReactNode;
};

function loadTrackingMode(storageKey: string): TrackingMode {
  const stored = localStorage.getItem(storageKey);

  if (stored === "manual" || stored === "qr") return stored;

  return "qr";
}

export function TrackingModeProvider({ children }: Props) {
  const { user } = useAuth();

  const storageKey =
    user !== null ? `inventory-tracking-mode:${user.id}` : null;

  const [trackingMode, setTrackingModeState] = useState<TrackingMode>(() => {
    if (storageKey === null) return "qr";
    return loadTrackingMode(storageKey);
  });

  if (user === null) return children;

  function setTrackingMode(mode: TrackingMode) {
    if (storageKey === null) return;

    localStorage.setItem(storageKey, mode);
    setTrackingModeState(mode);
  }

  return (
    <TrackingModeContext.Provider value={{ trackingMode, setTrackingMode }}>
      {children}
    </TrackingModeContext.Provider>
  );
}
