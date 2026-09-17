import { useEffect, useRef, type MouseEvent, type ReactNode } from "react";
import { X } from "lucide-react";

import styles from "./Dialog.module.css";

type Props = {
  title: ReactNode;
  description?: ReactNode;
  children: ReactNode;

  onClose: () => void;

  closeDisabled?: boolean;
  className?: string;
};

export function Dialog({
  title,
  description,
  children,
  onClose,
  closeDisabled = false,
  className,
}: Props) {
  const dialogRef = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    dialogRef.current?.showModal();
  }, []);

  function close() {
    if (!closeDisabled) {
      dialogRef.current?.close();
    }
  }

  function handleBackdropClick(event: MouseEvent<HTMLDialogElement>) {
    event.stopPropagation();

    if (event.target === dialogRef.current) {
      close();
    }
  }

  return (
    <dialog
      ref={dialogRef}
      className={`${styles.dialog} ${className ?? ""}`}
      onClose={(event) => {
        event.stopPropagation();
        onClose();
      }}
      onCancel={(event) => {
        event.stopPropagation();

        if (closeDisabled) {
          event.preventDefault();
        }
      }}
      onClick={handleBackdropClick}
    >
      <div className={styles.content}>
        <header className={styles.header}>
          <div>
            <h2 className={styles.title}>{title}</h2>

            {description !== undefined && (
              <p className={styles.description}>{description}</p>
            )}
          </div>

          <button
            type="button"
            className={styles.close}
            aria-label="Close"
            onClick={(event) => {
              event.stopPropagation();
              close();
            }}
            disabled={closeDisabled}
          >
            <X aria-hidden="true" />
          </button>
        </header>

        {children}
      </div>
    </dialog>
  );
}
