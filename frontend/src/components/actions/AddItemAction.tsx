import { Plus } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import styles from "./AddItemAction.module.css";

type Props = {
  onClick: () => void;
};

export function AddItemAction({ onClick }: Props) {
  const buttonRef = useRef<HTMLButtonElement>(null);
  const [showFloatingButton, setShowFloatingButton] = useState(false);

  useEffect(() => {
    const element = buttonRef.current;

    if (element === null) return;

    const observer = new IntersectionObserver(
      ([entry]) => {
        setShowFloatingButton(!entry.isIntersecting);
      },
      {
        threshold: 0,
      },
    );

    observer.observe(element);

    return () => observer.disconnect();
  }, []);

  return (
    <>
      <button
        ref={buttonRef}
        type="button"
        className="button button--primary"
        onClick={onClick}
      >
        Add item
      </button>

      {showFloatingButton && (
        <button
          type="button"
          className={styles.floating}
          onClick={onClick}
          aria-label="Add item"
        >
          <Plus />
        </button>
      )}
    </>
  );
}
