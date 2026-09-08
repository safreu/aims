import { Outlet } from "react-router-dom";
import { TrackingModeProvider } from "../tracking-mode/TrackingModeProvider";

export function SettingsProviderLayout() {
  return (
    <TrackingModeProvider>
      <Outlet />
    </TrackingModeProvider>
  );
}
