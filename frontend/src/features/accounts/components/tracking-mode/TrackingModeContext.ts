import { createContext, useContext } from "react";

export type TrackingMode = "qr" | "manual";

type TrackingModeContextValue = {
  trackingMode: TrackingMode;
  setTrackingMode: (mode: TrackingMode) => void;
};

export const TrackingModeContext =
  createContext<TrackingModeContextValue | null>(null);

export function useTrackingMode(): TrackingModeContextValue {
  const context = useContext(TrackingModeContext);

  if (context === null) {
    throw new Error("useTrackingMode must be used inside TrackingModeProvider");
  }

  return context;
}
