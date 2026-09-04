import { createContext, useContext } from "react";

type DangerModeContextValue = {
  dangerMode: boolean;
  setDangerMode: (enabled: boolean) => void;
};

export const DangerModeContext = createContext<DangerModeContextValue | null>(
  null,
);

export function useDangerMode() {
  const context = useContext(DangerModeContext);

  if (context === null) {
    throw new Error("useDangerMode must be used inside DangerModeProvider");
  }

  return context;
}
