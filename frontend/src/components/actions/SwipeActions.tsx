import {
  useRef,
  useState,
  type ElementType,
  type PointerEvent as ReactPointerEvent,
  type ReactNode,
} from "react";

import styles from "./SwipeActions.module.css";

const DEFAULT_THRESHOLD = 100;
const DEFAULT_MAX_DISTANCE = 140;
const DRAG_THRESHOLD = 8;

type SwipeActionVariant = "success" | "danger";

type Props = {
  children: ReactNode;

  onSwipeRight?: () => void | Promise<void>;
  onSwipeLeft?: () => void | Promise<void>;

  rightLabel?: string;
  leftLabel?: string;

  rightVariant?: SwipeActionVariant;
  leftVariant?: SwipeActionVariant;

  threshold?: number;
  maxDistance?: number;

  disabled?: boolean;
  as?: ElementType;
  className?: string;
};

export function SwipeActions({
  children,
  onSwipeRight,
  onSwipeLeft,
  rightLabel,
  leftLabel,
  rightVariant = "success",
  leftVariant = "danger",
  threshold = DEFAULT_THRESHOLD,
  maxDistance = DEFAULT_MAX_DISTANCE,
  disabled = false,
  as: Component = "div",
  className,
}: Props) {
  const startXRef = useRef<number | null>(null);
  const startYRef = useRef<number | null>(null);
  const pointerIdRef = useRef<number | null>(null);
  const directionRef = useRef<"left" | "right" | null>(null);

  const [offset, setOffset] = useState(0);
  const [isDragging, setIsDragging] = useState(false);

  function resetGesture() {
    startXRef.current = null;
    startYRef.current = null;
    pointerIdRef.current = null;
    directionRef.current = null;

    setOffset(0);
    setIsDragging(false);
  }

  function handlePointerDown(event: ReactPointerEvent<HTMLElement>) {
    if (disabled || event.pointerType === "mouse") {
      return;
    }

    startXRef.current = event.clientX;
    startYRef.current = event.clientY;
    pointerIdRef.current = event.pointerId;
    directionRef.current = null;
  }

  function handlePointerMove(event: ReactPointerEvent<HTMLElement>) {
    if (
      disabled ||
      startXRef.current === null ||
      startYRef.current === null ||
      pointerIdRef.current !== event.pointerId
    ) {
      return;
    }

    const deltaX = event.clientX - startXRef.current;
    const deltaY = event.clientY - startYRef.current;

    if (directionRef.current === null) {
      if (
        Math.abs(deltaX) < DRAG_THRESHOLD &&
        Math.abs(deltaY) < DRAG_THRESHOLD
      ) {
        return;
      }

      // The user is scrolling vertically rather than swiping.
      if (Math.abs(deltaY) >= Math.abs(deltaX)) {
        resetGesture();
        return;
      }

      if (deltaX > 0) {
        if (onSwipeRight === undefined) {
          resetGesture();
          return;
        }

        directionRef.current = "right";
      } else {
        if (onSwipeLeft === undefined) {
          resetGesture();
          return;
        }

        directionRef.current = "left";
      }

      setIsDragging(true);
    }

    const direction = directionRef.current;

    if (direction === "right") {
      setOffset(Math.min(Math.max(deltaX, 0), maxDistance));
      return;
    }

    if (direction === "left") {
      setOffset(Math.max(Math.min(deltaX, 0), -maxDistance));
    }
  }

  function handlePointerUp(event: ReactPointerEvent<HTMLElement>) {
    if (pointerIdRef.current !== event.pointerId) {
      return;
    }

    const direction = directionRef.current;

    const shouldTrigger =
      direction === "right"
        ? offset >= threshold
        : direction === "left"
          ? offset <= -threshold
          : false;

    resetGesture();

    if (!shouldTrigger || disabled) {
      return;
    }

    if (direction === "right") {
      void onSwipeRight?.();
    } else if (direction === "left") {
      void onSwipeLeft?.();
    }
  }

  function handlePointerCancel() {
    resetGesture();
  }

  return (
    <Component
      className={`${styles.container} ${className ?? ""}`}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={handlePointerUp}
      onPointerCancel={handlePointerCancel}
    >
      {rightLabel !== undefined && (
        <div
          className={`${styles.action} ${styles.actionRight} ${
            styles[rightVariant]
          }`}
          aria-hidden="true"
        >
          <span>{rightLabel}</span>
        </div>
      )}

      {leftLabel !== undefined && (
        <div
          className={`${styles.action} ${styles.actionLeft} ${
            styles[leftVariant]
          }`}
          aria-hidden="true"
        >
          <span>{leftLabel}</span>
        </div>
      )}

      <div
        className={`${styles.foreground} ${isDragging ? styles.dragging : ""}`}
        style={{ transform: `translateX(${offset}px)` }}
      >
        {children}
      </div>
    </Component>
  );
}
