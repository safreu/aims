import { useState, type SubmitEvent } from "react";

import { Dialog } from "../../../../components/dialog/Dialog";

type Props = {
  registering: boolean;
  onRegister: (name: string) => Promise<void>;
  onClose: () => void;
};

export function RegisterCurrentDeviceDialog({
  registering,
  onRegister,
  onClose,
}: Props) {
  const [name, setName] = useState("");

  function handleSubmit(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    const trimmedName = name.trim();

    if (trimmedName === "") return;

    void onRegister(trimmedName);
  }

  return (
    <Dialog
      title="Register this device"
      description="Register this phone so it can scan Aims QR codes."
      onClose={onClose}
      closeDisabled={registering}
    >
      <form className="dialog__form" onSubmit={handleSubmit}>
        <div className="dialog__field">
          <label htmlFor="device-name">Device name</label>

          <input
            id="device-name"
            type="text"
            value={name}
            onChange={(event) => setName(event.target.value)}
            placeholder="My phone"
            autoFocus
            disabled={registering}
          />

          <p className="dialog__hint">
            Choose a name that helps you recognize this device later
          </p>
        </div>

        <div className="dialog__actions">
          <button
            type="button"
            className="button button--secondary"
            onClick={onClose}
            disabled={registering}
          >
            Cancel
          </button>

          <button
            type="submit"
            className="button button--primary"
            disabled={registering || name.trim().length === 0}
          >
            {registering ? "Registering..." : "Register device"}
          </button>
        </div>
      </form>
    </Dialog>
  );
}
