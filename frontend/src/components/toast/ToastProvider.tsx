import {
  useCallback,
  useRef,
  useState,
  type PointerEvent,
  type PropsWithChildren,
} from "react";
import "./ToastProvider.css";
import { ToastContext, type ToastType } from "./ToastContext";

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
const SWIPE_DISMISS_DISTANCE = 60;
const TAP_MOVEMENT_THRESHOLD = 6;

export function ToastProvider({ children }: PropsWithChildren) {
  const toastContainerRef = useRef<HTMLDivElement>(null);
  const timersRef = useRef(new Map<number, ToastTimers>());
  const nextToastIdRef = useRef(0);

  const pointerRef = useRef<{
    toastId: number;
    pointerId: number;
    startX: number;
  } | null>(null);

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

  function handlePointerDown(
    event: PointerEvent<HTMLDivElement>,
    toastId: number,
  ) {
    pointerRef.current = {
      toastId,
      pointerId: event.pointerId,
      startX: event.clientX,
    };
  }

  function handlePointerUp(
    event: PointerEvent<HTMLDivElement>,
    toastId: number,
  ) {
    const pointer = pointerRef.current;

    pointerRef.current = null;

    if (
      pointer === null ||
      pointer.toastId !== toastId ||
      pointer.pointerId !== event.pointerId
    ) {
      return;
    }

    const distance = event.clientX - pointer.startX;
    const absoluteDistance = Math.abs(distance);

    if (
      absoluteDistance >= SWIPE_DISMISS_DISTANCE ||
      absoluteDistance <= TAP_MOVEMENT_THRESHOLD
    ) {
      dismissToast(toastId);
    }
  }

  function handlePointerCancel() {
    pointerRef.current = null;
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
            className={`toast toast--${toast.type} ${
              toast.closing ? "toast--closing" : ""
            }`}
            role="button"
            tabIndex={0}
            onPointerDown={(event) => handlePointerDown(event, toast.id)}
            onPointerUp={(event) => handlePointerUp(event, toast.id)}
            onPointerCancel={handlePointerCancel}
            onKeyDown={(event) => {
              if (event.key === "Enter" || event.key === " ") {
                event.preventDefault();
                dismissToast(toast.id);
              }
            }}
          >
            <span className="toast__icon" aria-hidden="true">
              {toastIcon(toast.type)}
            </span>

            <span className="toast__message">{toast.message}</span>

            {toast.count > 1 && (
              <span className="toast__count">×{toast.count}</span>
            )}

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
