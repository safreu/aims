import { Check, ChevronDown, Plus, Settings, User, Users } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";

import {
  DropdownMenu,
  DropdownMenuItem,
  DropdownMenuSeparator,
} from "../../../components/dropdown-menu/DropdownMenu";
import { useToast } from "../../../components/toast/ToastContext";
import { getHouseholds } from "../api";
import { useUserEvents } from "../events/UserEventsContext";
import type { Household } from "../types";
import { CreateHouseholdDialog } from "./CreateHouseholdDialog";
import styles from "./HouseholdSwitcher.module.css";

type HouseholdSwitcherProps = {
  householdId?: string;
};

export function HouseholdSwitcher({ householdId }: HouseholdSwitcherProps) {
  const navigate = useNavigate();
  const { showToast } = useToast();
  const { subscribe } = useUserEvents();

  const [households, setHouseholds] = useState<Household[]>([]);
  const [showCreateDialog, setShowCreateDialog] = useState(false);

  const refreshHouseholds = useCallback(async () => {
    const households = await getHouseholds();
    setHouseholds(households);
  }, []);

  useEffect(() => {
    async function loadHouseholds() {
      const households = await getHouseholds();
      setHouseholds(households);
    }

    void loadHouseholds().catch(() =>
      showToast("Failed to load households", "error"),
    );
  }, [showToast]);

  useEffect(() => {
    return subscribe("household_memberships_changed", () => {
      void refreshHouseholds().catch(() =>
        showToast("Failed to refresh households", "error"),
      );
    });
  }, [showToast, subscribe, refreshHouseholds]);

  const currentHousehold = households.find(
    (household) => household.id === householdId,
  );

  function handleSelectHousehold(id: string) {
    navigate(`/households/${id}/inventory`);
  }

  return (
    <>
      <DropdownMenu
        trigger={
          <button
            type="button"
            className={styles.trigger}
            aria-label="Select household"
          >
            {currentHousehold !== undefined && (
              <span className={styles.typeIcon}>
                {currentHousehold.kind === "shared" ? <Users /> : <User />}
              </span>
            )}

            <span className={styles.label}>
              {currentHousehold?.name ?? "Select household"}
            </span>

            <ChevronDown className={styles.chevron} />
          </button>
        }
      >
        {households.map((household) => (
          <DropdownMenuItem
            key={household.id}
            onSelect={() => handleSelectHousehold(household.id)}
          >
            <span className={styles.typeIcon}>
              {household.kind === "shared" ? <Users /> : <User />}
            </span>

            <span className={styles.name}>{household.name}</span>

            {household.id === householdId && (
              <Check className={styles.selected} />
            )}
          </DropdownMenuItem>
        ))}

        {households.length > 0 && <DropdownMenuSeparator />}

        <DropdownMenuItem
          className={styles.create}
          onSelect={() => setShowCreateDialog(true)}
        >
          <Plus />
          <span>Create household</span>
        </DropdownMenuItem>

        {householdId !== undefined && (
          <DropdownMenuItem
            onSelect={() => navigate(`/households/${householdId}/settings`)}
          >
            <Settings />
            <span>Manage household</span>
          </DropdownMenuItem>
        )}
      </DropdownMenu>

      {showCreateDialog && (
        <CreateHouseholdDialog
          onCreated={refreshHouseholds}
          onClose={() => setShowCreateDialog(false)}
        />
      )}
    </>
  );
}
