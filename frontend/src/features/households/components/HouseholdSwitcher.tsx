import { useNavigate } from "react-router-dom";
import { useToast } from "../../../components/toast/ToastContext";
import { useEffect, useState } from "react";
import type { Household } from "../types";
import { getHouseholds } from "../api";
import {
  DropdownMenu,
  DropdownMenuItem,
  DropDownMenuSeparator,
} from "../../../components/dropdown-menu/DropdownMenu";
import { Check, ChevronDown, Plus, Settings, User, Users } from "lucide-react";

import "./HouseholdSwitcher.css";
import { CreateHouseholdDialog } from "./CreateHouseholdDialog";

type HouseholdSwitcherProps = {
  householdId?: string;
};

export function HouseholdSwitcher({ householdId }: HouseholdSwitcherProps) {
  const navigate = useNavigate();
  const { showToast } = useToast();

  const [households, setHouseholds] = useState<Household[]>([]);
  const [showCreateDialog, setShowCreateDialog] = useState(false);

  async function refreshHouseholds() {
    const households = await getHouseholds();
    setHouseholds(households);
  }

  useEffect(() => {
    async function loadHouseholds() {
      const households = await getHouseholds();
      setHouseholds(households);
    }

    void loadHouseholds().catch(() =>
      showToast("Failed to load households", "error"),
    );
  }, [showToast]);

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
            className="household-switcher"
            aria-label="Select household"
          >
            {currentHousehold !== undefined && (
              <span className="household-switcher__type-icon">
                {currentHousehold.kind === "shared" ? <Users /> : <User />}
              </span>
            )}

            <span className="household-switcher__label">
              {currentHousehold?.name ?? "Select household"}
            </span>

            <ChevronDown className="household-switcher__chevron" />
          </button>
        }
      >
        {households.map((household) => (
          <DropdownMenuItem
            key={household.id}
            onSelect={() => handleSelectHousehold(household.id)}
          >
            <span className="household-switcher__type-icon">
              {household.kind === "shared" ? <Users /> : <User />}
            </span>

            <span className="household-switcher__name">{household.name}</span>

            {household.id === householdId && (
              <Check className="household-switcher__selected" />
            )}
          </DropdownMenuItem>
        ))}

        {households.length > 0 && <DropDownMenuSeparator />}

        <DropdownMenuItem
          className="household-switcher__create"
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
