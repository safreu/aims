import { useState, type ReactNode } from "react";
import { DangerModeContext } from "./DangerModeContext";

type Props = {
  children: ReactNode;
};

export function DangerModeProvider({ children }: Props) {
  const [dangerMode, setDangerMode] = useState(false);

  return (
    <DangerModeContext.Provider value={{ dangerMode, setDangerMode }}>
      {children}
    </DangerModeContext.Provider>
  );
}
