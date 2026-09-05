import { useDangerMode } from "../../features/households/danger-mode/DangerModeContext";
import { DropdownMenuCheckboxItem } from "./DropdownMenu";

export function DangerModeMenuItem() {
  const { dangerMode, setDangerMode } = useDangerMode();

  return (
    <DropdownMenuCheckboxItem
      checked={dangerMode}
      onCheckedChange={setDangerMode}
    >
      <div className="dropdown-menu__item-content">
        <span>Manual stock controls</span>
        <span>{dangerMode ? "ON" : "OFF"}</span>
      </div>
    </DropdownMenuCheckboxItem>
  );
}
