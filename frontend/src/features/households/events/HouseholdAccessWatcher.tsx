import { useNavigate } from "react-router-dom";
import { useHouseholdEvents } from "./HouseholdEventsContext";
import { useToast } from "../../../components/toast/ToastContext";
import { useEffect } from "react";
import { getHousehold } from "../api";
import { isHouseholdAccessError } from "../errors";

export function HouseholdAccessWatcher({
  householdId,
}: {
  householdId: string;
}) {
  const { subscribe } = useHouseholdEvents();
  const navigate = useNavigate();
  const { showToast } = useToast();

  useEffect(() => {
    return subscribe("household_changed", () => {
      void getHousehold(householdId).catch((error) => {
        if (isHouseholdAccessError(error)) {
          navigate("/households");
        }
      });
    });
  }, [subscribe, householdId, showToast, navigate]);

  return null;
}
