import { NavLink, Outlet, useParams } from "react-router-dom";
import "./HouseholdLayout.css";
import { Boxes, ShoppingCart } from "lucide-react";
import { AppHeader } from "../../../components/layout/AppHeader";
import { HouseholdEventsProvider } from "../events/HouseholdEventsProvider";
import { CategoryProvider } from "../../inventory/components/categories/CategoryProvider";
import { DangerModeProvider } from "../danger-mode/DangerModeProvider";

export function HouseholdLayout() {
  const { householdId } = useParams();

  if (householdId === undefined) {
    throw new Error("HouseholdLayout requires a HouseholdId");
  }

  const navigationClassName = ({ isActive }: { isActive: boolean }) =>
    `household-navigation__link${isActive ? " active" : ""}`;

  return (
    <DangerModeProvider>
      <HouseholdEventsProvider householdId={householdId}>
        <CategoryProvider householdId={householdId}>
          <div className="household-layout">
            <AppHeader householdId={householdId} />

            <main className="household-content">
              <Outlet />
            </main>

            <nav
              className="household-navigation"
              aria-label="Household navigation"
            >
              <div className="household-navigation__container">
                <NavLink
                  to={`/households/${householdId}/inventory`}
                  className={navigationClassName}
                >
                  <Boxes className="household-navigation__icon" />
                  <span>Inventory</span>
                </NavLink>

                <NavLink
                  to={`/households/${householdId}/shopping`}
                  className={navigationClassName}
                >
                  <ShoppingCart className="household-navigation__icon" />
                  <span>Shopping</span>
                </NavLink>
              </div>
            </nav>
          </div>
        </CategoryProvider>
      </HouseholdEventsProvider>
    </DangerModeProvider>
  );
}
