import { useState, type SubmitEvent } from "react";

import { Dialog } from "../../../../components/dialog/Dialog";
import {
  Select,
  type SelectOption,
} from "../../../../components/select/Select";
import type { DeviceKind } from "../../types";

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
  const [name, setName] = useState("");
  const [kind, setKind] = useState<DeviceKind>("scanner");

  function handleSubmit(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    const trimmedName = name.trim();

    if (trimmedName === "") return;

    void onRegister(trimmedName, kind);
  }

  return (
    <Dialog
      title="Register device"
      description="Add another device to this household"
      onClose={onClose}
      closeDisabled={registering}
    >
      <form className="dialog__form" onSubmit={handleSubmit}>
        <div className="dialog__field">
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

        <div className="dialog__field">
          <label>Device type</label>

          <Select
            value={kind}
            options={deviceKindOptions}
            onValueChange={(value) => setKind(value as DeviceKind)}
            disabled={registering}
            ariaLabel="Device type"
            portal={false}
          />
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
            disabled={registering || name.trim() === ""}
          >
            {registering ? "Registering..." : "Register"}
          </button>
        </div>
      </form>
    </Dialog>
  );
}
