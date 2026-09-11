import { useEffect, useRef, useState, type SubmitEvent } from "react";
import "./RegisterOtherDeviceDialog.css";
import type { DeviceKind } from "../../types";
import {
  Select,
  type SelectOption,
} from "../../../../components/select/Select";

type Props = {
  registering: boolean;
  onRegister: (name: string, kind: DeviceKind) => Promise<void>;
  onClose: () => void;
};

const deviceKindOptions: SelectOption<DeviceKind>[] = [
  { value: "scanner", label: "Scanner" },
  { value: "display", label: "Display" },
  { value: "other", label: "Other" },
];

export function RegisterOtherDeviceDialog({
  registering,
  onRegister,
  onClose,
}: Props) {
  const dialogRef = useRef<HTMLDialogElement>(null);

  const [name, setName] = useState("");
  const [kind, setKind] = useState<DeviceKind>("scanner");

  useEffect(() => {
    dialogRef.current?.showModal();
  }, []);

  function handleSubmit(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    const trimmedName = name.trim();

    if (trimmedName === "") return;

    void onRegister(trimmedName, kind);
  }

  return (
    <dialog
      ref={dialogRef}
      className="register-other-device-dialog"
      onClose={onClose}
      onClick={(event) => {
        if (event.target === dialogRef.current && !registering) {
          dialogRef.current?.close();
        }
      }}
    >
      <form
        className="register-other-device-dialog__content"
        onSubmit={handleSubmit}
      >
        <div className="register-other-device-dialog__header">
          <h2>Register device</h2>
          <p>Add another device to this household</p>
        </div>

        <div className="register-other-device-dialog__field">
          <label htmlFor="other-device-name">Device name</label>

          <input
            id="other-device-name"
            type="text"
            value={name}
            onChange={(event) => setName(event.target.value)}
            autoFocus
            disabled={registering}
          />
        </div>

        <div className="register-other-device-dialog__field">
          <label>Device type</label>

          <Select
            value={kind}
            options={deviceKindOptions}
            onValueChange={(event) => setKind(event as DeviceKind)}
            disabled={registering}
            ariaLabel="Device type"
            portal={false}
          />
        </div>

        <div className="register-other-device-dialog__actions">
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
            disabled={registering || name.trim() === ""}
          >
            {registering ? "Registering..." : "Register"}
          </button>
        </div>
      </form>
    </dialog>
  );
}
