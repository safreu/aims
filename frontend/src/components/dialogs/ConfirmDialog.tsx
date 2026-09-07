import { useEffect, useRef } from "react";
import "./ConfirmDialog.css";

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
      className="confirm-dialog"
      onCancel={(event) => {
        event.preventDefault();
        if (!loading) onCancel();
      }}
    >
      <div className="confirm-dialog__content">
        <h2>{title}</h2>

        <p>{description}</p>

        <div className="confirm-dialog__actions">
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
