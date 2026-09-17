import { useCallback, useRef, useState, type PropsWithChildren } from "react";

import { ToastContext, type ToastType } from "./ToastContext";
import styles from "./ToastProvider.module.css";
import { SwipeActions } from "../actions/SwipeActions";

type Toast = {
  id: number;
  message: string;
  type: ToastType;
  count: number;
  closing: boolean;
};

type ToastTimers = {
  closing?: number;
  removal?: number;
};

const MAX_VISIBLE_TOASTS = 3;
const TOAST_DURATION_MS = 4000;
const TOAST_EXIT_DURATION_MS = 300;

export function ToastProvider({ children }: PropsWithChildren) {
  const toastContainerRef = useRef<HTMLDivElement>(null);
  const timersRef = useRef(new Map<number, ToastTimers>());
  const nextToastIdRef = useRef(0);

  const [toasts, setToasts] = useState<Toast[]>([]);

  const clearToastTimers = useCallback((id: number) => {
    const timers = timersRef.current.get(id);

    if (timers?.closing !== undefined) {
      window.clearTimeout(timers.closing);
    }

    if (timers?.removal !== undefined) {
      window.clearTimeout(timers.removal);
    }

    timersRef.current.delete(id);
  }, []);

  const removeToast = useCallback(
    (id: number) => {
      clearToastTimers(id);

      setToasts((current) => {
        const remaining = current.filter((toast) => toast.id !== id);

        if (remaining.length === 0) {
          toastContainerRef.current?.hidePopover();
        }

        return remaining;
      });
    },
    [clearToastTimers],
  );

  const dismissToast = useCallback(
    (id: number) => {
      clearToastTimers(id);

      setToasts((current) =>
        current.map((toast) =>
          toast.id === id ? { ...toast, closing: true } : toast,
        ),
      );

      const removal = window.setTimeout(() => {
        removeToast(id);
      }, TOAST_EXIT_DURATION_MS);

      timersRef.current.set(id, { removal });
    },
    [clearToastTimers, removeToast],
  );

  const scheduleToastDismissal = useCallback(
    (id: number) => {
      clearToastTimers(id);

      const closing = window.setTimeout(() => {
        setToasts((current) =>
          current.map((toast) =>
            toast.id === id ? { ...toast, closing: true } : toast,
          ),
        );

        const removal = window.setTimeout(() => {
          removeToast(id);
        }, TOAST_EXIT_DURATION_MS);

        timersRef.current.set(id, { removal });
      }, TOAST_DURATION_MS - TOAST_EXIT_DURATION_MS);

      timersRef.current.set(id, { closing });
    },
    [clearToastTimers, removeToast],
  );

  const showToast = useCallback(
    (message: string, type: ToastType = "info") => {
      let toastId: number | null = null;
      let removedToastId: number | null = null;

      setToasts((current) => {
        const existing = current.find(
          (toast) =>
            toast.message === message && toast.type === type && !toast.closing,
        );

        if (existing !== undefined) {
          toastId = existing.id;

          return current.map((toast) =>
            toast.id === existing.id
              ? {
                  ...toast,
                  count: toast.count + 1,
                }
              : toast,
          );
        }

        nextToastIdRef.current += 1;
        const id = nextToastIdRef.current;
        toastId = id;

        const next = [
          ...current,
          {
            id,
            message,
            type,
            count: 1,
            closing: false,
          },
        ];

        if (next.length > MAX_VISIBLE_TOASTS) {
          removedToastId = next[0].id;
          return next.slice(-MAX_VISIBLE_TOASTS);
        }

        return next;
      });

      if (removedToastId !== null) {
        clearToastTimers(removedToastId);
      }

      if (toastId !== null) {
        scheduleToastDismissal(toastId);
      }

      toastContainerRef.current?.showPopover();
    },
    [clearToastTimers, scheduleToastDismissal],
  );

  return (
    <ToastContext.Provider value={{ showToast }}>
      {children}

      <div
        ref={toastContainerRef}
        className={styles.container}
        popover="manual"
        aria-live="polite"
      >
        {toasts.map((toast) => (
          <SwipeActions
            threshold={60}
            onSwipeRight={() => dismissToast(toast.id)}
            onSwipeLeft={() => dismissToast(toast.id)}
            disabled={toast.closing}
          >
            <div
              className={`${styles.toast} ${styles[toast.type]} ${
                toast.closing ? styles.closing : ""
              }`}
              role="button"
              tabIndex={0}
              onClick={() => dismissToast(toast.id)}
              onKeyDown={(event) => {
                if (event.key === "Enter" || event.key === " ") {
                  event.preventDefault();
                  dismissToast(toast.id);
                }
              }}
            >
              <span className={styles.icon} aria-hidden="true">
                {toastIcon(toast.type)}
              </span>

              <span className={styles.message}>{toast.message}</span>

              {toast.count > 1 && (
                <span className={styles.count}>×{toast.count}</span>
              )}

              <div className={styles.progress} />
            </div>
          </SwipeActions>
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
