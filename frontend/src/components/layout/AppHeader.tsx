import { Link } from "react-router-dom";
import "./AppHeader.css";
import { HouseholdSwitcher } from "../../features/households/components/HouseholdSwitcher";
import {
  DropdownMenu,
  DropdownMenuItem,
  DropDownMenuSeparator,
} from "../dropdown-menu/DropdownMenu";
import { ChevronDown, LogOut, User } from "lucide-react";
import { useAuth } from "../../features/auth/context/AuthContext";
import { DangerModeMenuItem } from "../dropdown-menu/DangerModeMenuItem";

type AppHeaderProps = {
  householdId?: string;
};

export function AppHeader({ householdId }: AppHeaderProps) {
  const { logout } = useAuth();

  function handleLogout() {
    void logout();
  }

  return (
    <header className="app-header">
      <div className="app-header__container">
        <div className="app-header__left">
          <Link to="/households" className="app-header__brand">
            Aims
          </Link>

          <HouseholdSwitcher householdId={householdId} />
        </div>

        <div className="app-header__actions">
          <DropdownMenu
            trigger={
              <button type="button" className="app-header__account-button">
                <User />
                <ChevronDown />
              </button>
            }
          >
            <DangerModeMenuItem />
            <DropDownMenuSeparator />

            <DropdownMenuItem onSelect={handleLogout}>
              <LogOut />
              <span>Logout</span>
            </DropdownMenuItem>
          </DropdownMenu>
        </div>
      </div>
    </header>
  );
}
