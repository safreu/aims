import { useState, type SubmitEvent } from "react";

import { Dialog } from "../../../../components/dialog/Dialog";
import type { Device } from "../../types";

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
  const [name, setName] = useState(device.name);

  function handleSubmit(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    const trimmedName = name.trim();

    if (trimmedName === "" || trimmedName === device.name) return;

    void onRename(trimmedName);
  }

  return (
    <Dialog title="Rename device" onClose={onClose} closeDisabled={renaming}>
      <form className="dialog__form" onSubmit={handleSubmit}>
        <div className="dialog__field">
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

        <div className="dialog__actions">
          <button
            type="button"
            className="button button--secondary"
            onClick={onClose}
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
    </Dialog>
  );
}
