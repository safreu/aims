import { Dialog } from "../dialog/Dialog";

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
  if (!open) {
    return null;
  }

  return (
    <Dialog
      title={title}
      description={description}
      onClose={onCancel}
      closeDisabled={loading}
    >
      <div className="dialog__actions">
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
    </Dialog>
  );
}
