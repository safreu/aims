import { useTrackingMode } from "../../features/accounts/components/tracking-mode/TrackingModeContext";
import { useDangerMode } from "../../features/households/danger-mode/DangerModeContext";
import {
  DropdownMenuCheckboxItem,
  DropdownMenuItemContent,
} from "./DropdownMenu";

export function DangerModeMenuItem() {
  const { dangerMode, setDangerMode } = useDangerMode();
  const { trackingMode } = useTrackingMode();

  if (trackingMode === "manual") return null;

  return (
    <DropdownMenuCheckboxItem
      checked={dangerMode}
      onCheckedChange={setDangerMode}
    >
      <DropdownMenuItemContent>
        <span>Manual stock controls</span>
        <span>{dangerMode ? "ON" : "OFF"}</span>
      </DropdownMenuItemContent>
    </DropdownMenuCheckboxItem>
  );
}
