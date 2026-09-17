import { ChevronDown, LogOut, User } from "lucide-react";
import { Link, useNavigate } from "react-router-dom";

import { useAuth } from "../../features/auth/context/AuthContext";
import { HouseholdSwitcher } from "../../features/households/components/HouseholdSwitcher";
import { DangerModeMenuItem } from "../dropdown-menu/DangerModeMenuItem";
import {
  DropdownMenu,
  DropdownMenuItem,
  DropdownMenuSeparator,
} from "../dropdown-menu/DropdownMenu";
import styles from "./AppHeader.module.css";

type AppHeaderProps = {
  householdId?: string;
};

export function AppHeader({ householdId }: AppHeaderProps) {
  const { logout } = useAuth();
  const navigate = useNavigate();

  function handleLogout() {
    void logout();
  }

  return (
    <header className={styles.header}>
      <div className={styles.container}>
        <div className={styles.left}>
          <Link to="/households" className={styles.brand}>
            Aims
          </Link>

          <HouseholdSwitcher householdId={householdId} />
        </div>

        <div className={styles.actions}>
          <DropdownMenu
            trigger={
              <button
                type="button"
                className={styles.accountButton}
                aria-label="Account menu"
              >
                <User />
                <ChevronDown />
              </button>
            }
          >
            <DropdownMenuItem onSelect={() => navigate("/account")}>
              <User />
              <span>Account settings</span>
            </DropdownMenuItem>

            <DropdownMenuSeparator />

            {householdId !== undefined && (
              <>
                <DangerModeMenuItem />
                <DropdownMenuSeparator />
              </>
            )}

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
