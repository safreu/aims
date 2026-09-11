import { useEffect, useRef, useState, type SubmitEvent } from "react";
import "./RegisterCurrentDeviceDialog.css";

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
  const dialogRef = useRef<HTMLDialogElement>(null);
  const [name, setName] = useState("");

  useEffect(() => {
    dialogRef.current?.showModal();
  }, []);

  function handleSubmit(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    const trimmedName = name.trim();

    if (trimmedName === "") return;

    void onRegister(trimmedName);
  }

  return (
    <dialog
      ref={dialogRef}
      className="register-device-dialog"
      onClose={onClose}
      onClick={(event) => {
        if (event.target === dialogRef.current && !registering) {
          dialogRef.current?.close();
        }
      }}
    >
      <form className="register-device-dialog__content" onSubmit={handleSubmit}>
        <div className="register-device-dialog__header">
          <div>
            <h2>Register this device</h2>
            <p>Register this phone so it can scan Aims QR codes</p>
          </div>

          <button
            type="button"
            className="register-device-dialog__close"
            onClick={() => dialogRef.current?.close()}
            disabled={registering}
            aria-label="Close"
          >
            ×
          </button>
        </div>

        <div className="register-device-dialog__field">
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

          <p>Choose a name that helps you recognize this device later</p>
        </div>

        <div className="register-device-dialog__actions">
          <button
            type="button"
            className="button button--secondary"
            onClick={() => dialogRef.current?.close()}
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
    </dialog>
  );
}
