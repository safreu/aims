import { useEffect, useRef, useState, type SubmitEvent } from "react";

import styles from "./RegisterCurrentDeviceDialog.module.css";

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
      className="dialog"
      onClose={onClose}
      onClick={(event) => {
        if (event.target === dialogRef.current && !registering) {
          dialogRef.current?.close();
        }
      }}
    >
      <form className="dialog__content" onSubmit={handleSubmit}>
        <div className={`dialog__header ${styles.header}`}>
          <div className={styles.headerText}>
            <h2 className="dialog__title">Register this device</h2>

            <p className="dialog__description">
              Register this phone so it can scan Aims QR codes.
            </p>
          </div>

          <button
            type="button"
            className={styles.close}
            onClick={() => dialogRef.current?.close()}
            disabled={registering}
            aria-label="Close"
          >
            ×
          </button>
        </div>

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
