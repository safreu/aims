import { useEffect, useRef, useState, type SubmitEvent } from "react";
import type { Device } from "../../types";
import "./RenameDeviceDialog.css";

type Props = {
  device: Device;
  renaming: boolean;
  onRename: (name: string) => Promise<void>;
  onClose: () => void;
};

export function RenameDeviceDialog({
  device,
  renaming,
  onRename,
  onClose,
}: Props) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const [name, setName] = useState(device.name);

  useEffect(() => {
    dialogRef.current?.showModal();
  }, []);

  function handleSubmit(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    const trimmedName = name.trim();

    if (trimmedName === "" || trimmedName === device.name) return;

    void onRename(trimmedName);
  }

  return (
    <dialog
      ref={dialogRef}
      className="rename-device-dialog"
      onClose={onClose}
      onClick={(event) => {
        if (event.target === dialogRef.current && !renaming) {
          dialogRef.current?.close();
        }
      }}
    >
      <form className="rename-device-dialog__content" onSubmit={handleSubmit}>
        <div className="rename-device-dialog__header">
          <div>
            <h2>Rename device</h2>
          </div>
        </div>

        <div className="rename-device-dialog__field">
          <label htmlFor="device-name">Device name</label>

          <input
            id="device-name"
            type="text"
            value={name}
            onChange={(event) => setName(event.target.value)}
            autoFocus
            disabled={renaming}
          />
        </div>

        <div className="rename-device-dialog__actions">
          <button
            type="button"
            className="button button--secondary"
            onClick={() => dialogRef.current?.close()}
            disabled={renaming}
          >
            Cancel
          </button>

          <button
            type="submit"
            className="button button--primary"
            disabled={
              renaming || name.trim() === "" || name.trim() === device.name
            }
          >
            {renaming ? "Saving..." : "Save"}
          </button>
        </div>
      </form>
    </dialog>
  );
}
