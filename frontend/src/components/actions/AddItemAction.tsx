import { Plus } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import "./AddItemAction.css";

type Props = {
  onClick: () => void;
};

export function AddItemAction({ onClick }: Props) {
  const buttonRef = useRef<HTMLDivElement>(null);
  const [showFloatingButton, setShowFloatingButton] = useState(false);

  useEffect(() => {
    const element = buttonRef.current;

    if (!element) return;

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
      <div ref={buttonRef}>
        <button
          type="button"
          className="button button--primary"
          onClick={onClick}
        >
          Add item
        </button>

        {showFloatingButton && (
          <button
            type="button"
            className="add-item-action__floating"
            onClick={onClick}
            aria-label="Add item"
          >
            <Plus />
          </button>
        )}
      </div>
    </>
  );
}
