import { createContext, useContext } from "react";

export type ToastType = "error" | "warning" | "info" | "success";

type ToastContextValue = {
  showToast: (message: string, type?: ToastType) => void;
};

export const ToastContext = createContext<ToastContextValue | null>(null);

export function useToast() {
  const context = useContext(ToastContext);

  if (context === null) {
    throw new Error("useToast must be used inside ToastProvider");
  }

  return context;
}
