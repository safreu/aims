import { Link } from "react-router-dom";
import { LogoutButton } from "../../features/auth/components/LogoutButton";
import "./AppHeader.css";
import { HouseholdSwitcher } from "../../features/households/components/HouseholdSwitcher";

type AppHeaderProps = {
  householdId?: string;
};

export function AppHeader({ householdId }: AppHeaderProps) {
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
          <LogoutButton />
        </div>
      </div>
    </header>
  );
}
