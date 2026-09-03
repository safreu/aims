import { useRef, useState, type PropsWithChildren } from "react";
import "./ToastProvider.css";
import { ToastContext, type ToastType } from "./ToastContext";

type Toast = {
  id: number;
  message: string;
  type: ToastType;
  closing: boolean;
};

export function ToastProvider({ children }: PropsWithChildren) {
  const toastContainerRef = useRef<HTMLDivElement>(null);

  const [toasts, setToasts] = useState<Toast[]>([]);

  function showToast(message: string, type: ToastType = "info") {
    const id = Date.now();

    setToasts((current) => [...current, { id, message, closing: false, type }]);

    toastContainerRef.current?.showPopover();

    window.setTimeout(() => {
      setToasts((current) =>
        current.map((toast) =>
          toast.id === id ? { ...toast, closing: true } : toast,
        ),
      );
    }, 3700);

    window.setTimeout(() => {
      setToasts((current) => {
        const remaining = current.filter((toast) => toast.id !== id);

        if (remaining.length === 0) {
          toastContainerRef.current?.hidePopover();
        }

        return remaining;
      });
    }, 4000);
  }

  return (
    <ToastContext.Provider value={{ showToast }}>
      {children}

      <div
        ref={toastContainerRef}
        className="toast-container"
        popover="manual"
        aria-live="polite"
      >
        {toasts.map((toast) => (
          <div
            key={toast.id}
            className={`toast toast--${toast.type} ${toast.closing ? "toast--closing" : ""}`}
          >
            <span className="toast__icon" aria-hidden="true">
              {toastIcon(toast.type)}
            </span>
            <span className="toast__message">{toast.message}</span>

            <div className="toast__progress" />
          </div>
        ))}
      </div>
    </ToastContext.Provider>
  );
}

function toastIcon(type: ToastType) {
  switch (type) {
    case "error":
      return "×";
    case "warning":
      return "!";
    case "info":
      return "i";
    case "success":
      return "✓";
  }
}
