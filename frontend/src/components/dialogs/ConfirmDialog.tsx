import { useEffect, useRef } from "react";

import styles from "./ConfirmDialog.module.css";

type Props = {
  open: boolean;
  title: string;
  description: string;
  confirmLabel: string;
  cancelLabel?: string;
  destructive?: boolean;
  loading?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
};

export function ConfirmDialog({
  open,
  title,
  description,
  confirmLabel,
  cancelLabel = "Cancel",
  destructive = false,
  loading = false,
  onConfirm,
  onCancel,
}: Props) {
  const dialogRef = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    const dialog = dialogRef.current;

    if (dialog === null) return;

    if (open && !dialog.open) dialog.showModal();

    if (!open && dialog.open) dialog.close();
  }, [open]);

  return (
    <dialog
      ref={dialogRef}
      className={styles.dialog}
      onCancel={(event) => {
        event.preventDefault();

        if (!loading) onCancel();
      }}
    >
      <div className={styles.content}>
        <h2 className={styles.title}>{title}</h2>

        <p className={styles.description}>{description}</p>

        <div className={styles.actions}>
          <button
            type="button"
            className="button button--ghost"
            onClick={onCancel}
            disabled={loading}
          >
            {cancelLabel}
          </button>

          <button
            type="button"
            className={
              destructive ? "button button--danger" : "button button--primary"
            }
            onClick={onConfirm}
            disabled={loading}
          >
            {loading ? "Please wait" : confirmLabel}
          </button>
        </div>
      </div>
    </dialog>
  );
}
