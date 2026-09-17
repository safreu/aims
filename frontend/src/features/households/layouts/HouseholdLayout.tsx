import type { ReactNode } from "react";
import { Boxes, ScanLine, ShoppingCart } from "lucide-react";
import { NavLink, Outlet, useParams } from "react-router-dom";

import { AppHeader } from "../../../components/layout/AppHeader";
import { useTrackingMode } from "../../accounts/components/tracking-mode/TrackingModeContext";
import { CategoryProvider } from "../../inventory/components/categories/CategoryProvider";
import { DangerModeProvider } from "../danger-mode/DangerModeProvider";
import { HouseholdEventsProvider } from "../events/HouseholdEventsProvider";

import styles from "./HouseholdLayout.module.css";

export function HouseholdLayout() {
  const { householdId } = useParams();
  const { trackingMode } = useTrackingMode();

  if (householdId === undefined) {
    throw new Error("HouseholdLayout requires a householdId");
  }

  function navigationClassName({ isActive }: { isActive: boolean }) {
    return `${styles.navigationLink}${
      isActive ? ` ${styles.navigationLinkActive}` : ""
    }`;
  }

  function scanClassName({ isActive }: { isActive: boolean }) {
    return `${styles.scan}${isActive ? ` ${styles.scanActive}` : ""}`;
  }

  return (
    <HouseholdProviders key={householdId} householdId={householdId}>
      <div className={styles.layout}>
        <AppHeader householdId={householdId} />

        <main className={styles.content}>
          <Outlet />
        </main>

        <nav className={styles.navigation} aria-label="Household navigation">
          <div className={styles.navigationContainer}>
            <NavLink
              to={`/households/${householdId}/inventory`}
              className={navigationClassName}
            >
              <Boxes className={styles.navigationIcon} aria-hidden="true" />

              <span>Inventory</span>
            </NavLink>

            {trackingMode === "qr" && (
              <NavLink
                to={`/households/${householdId}/scanner`}
                className={scanClassName}
                aria-label="Scan QR code"
              >
                <ScanLine className={styles.scanIcon} aria-hidden="true" />
              </NavLink>
            )}

            <NavLink
              to={`/households/${householdId}/shopping`}
              className={navigationClassName}
            >
              <ShoppingCart
                className={styles.navigationIcon}
                aria-hidden="true"
              />

              <span>Shopping</span>
            </NavLink>
          </div>
        </nav>
      </div>
    </HouseholdProviders>
  );
}

type HouseholdProvidersProps = {
  householdId: string;
  children: ReactNode;
};

function HouseholdProviders({
  householdId,
  children,
}: HouseholdProvidersProps) {
  return (
    <DangerModeProvider>
      <HouseholdEventsProvider householdId={householdId}>
        <CategoryProvider householdId={householdId}>
          {children}
        </CategoryProvider>
      </HouseholdEventsProvider>
    </DangerModeProvider>
  );
}
