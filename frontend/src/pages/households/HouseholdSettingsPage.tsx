import { useCallback, useEffect } from "react";
import "./HouseholdSettingsPage.css";
import { useParams } from "react-router-dom";
import {
  getHousehold,
  getHouseholdMembers,
} from "../../features/households/api";
import { useAuth } from "../../features/auth/context/AuthContext";
import { useHouseholdEvents } from "../../features/households/events/HouseholdEventsContext";
import { DeviceList } from "../../features/devices/components/DeviceList";
import { useTrackingMode } from "../../features/accounts/components/tracking-mode/TrackingModeContext";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { queryKeys } from "../../api/queryKeys";
import Skeleton from "react-loading-skeleton";
import { GeneralHouseholdSettings } from "../../features/households/components/settings/GeneralHouseholdSettings";
import { HouseholdMembersSettings } from "../../features/households/components/settings/HouseholdMembersSettings";
import { HouseholdDangerZone } from "../../features/households/components/settings/HouseholdDangerMode";

export function HouseholdSettingsPage() {
  const { householdId } = useParams();
  const { user } = useAuth();
  const { subscribe } = useHouseholdEvents();
  const { trackingMode } = useTrackingMode();

  if (householdId === undefined) {
    throw new Error("HouseholdSettingsPage requires a householdId");
  }

  const resolvedHousehold = householdId;

  const queryClient = useQueryClient();

  const {
    data: household,
    isPending: isHouseholdPending,
    isError: isHouseholdError,
  } = useQuery({
    queryKey: queryKeys.household.detail(resolvedHousehold),
    queryFn: () => getHousehold(resolvedHousehold),
  });

  const {
    data: members = [],
    isPending: areMembersPending,
    isError: areMembersError,
  } = useQuery({
    queryKey: queryKeys.household.members(resolvedHousehold),
    queryFn: () => getHouseholdMembers(resolvedHousehold),
  });

  const currentMember = members.find((member) => member.user_id == user?.id);

  const currentUserIsOwner = currentMember?.role === "owner";

  const hasOtherMembers = members.some((member) => member.user_id !== user?.id);

  const refreshHouseholdSettings = useCallback(async () => {
    return queryClient.invalidateQueries({
      queryKey: queryKeys.household.detail(resolvedHousehold),
    });
  }, [resolvedHousehold, queryClient]);

  const refreshMembers = useCallback(() => {
    return queryClient.invalidateQueries({
      queryKey: queryKeys.household.members(resolvedHousehold),
    });
  }, [queryClient, resolvedHousehold]);

  useEffect(() => {
    return subscribe("household_changed", () => {
      void refreshHouseholdSettings();
    });
  }, [subscribe, refreshHouseholdSettings]);

  const isPending = isHouseholdPending || areMembersPending;
  const isError = isHouseholdError || areMembersError;

  return (
    <main className="household-settings-page">
      <header className="household-settings-page__header">
        <h1>Household settings</h1>
        <p>
          {currentUserIsOwner
            ? "Change the name of this household"
            : "General information about this household"}
        </p>
      </header>

      {isPending ? (
        <HouseholdSettingsSkeleton />
      ) : isError ? (
        <p> Failed to load household settings</p>
      ) : (
        <>
          <GeneralHouseholdSettings
            householdId={resolvedHousehold}
            householdName={household?.name ?? ""}
            currentUserIsOwner={currentUserIsOwner}
            onChanged={refreshHouseholdSettings}
          />

          <HouseholdMembersSettings
            householdId={resolvedHousehold}
            householdKind={household!.kind}
            members={members}
            currentUserIsOwner={currentUserIsOwner}
            onChanged={refreshMembers}
          />

          {trackingMode === "qr" && (
            <section className="household-settings-page__section">
              <header className="household-settings-page__section-header">
                <h2>Devices</h2>
                <p>Manage devices connected to this household</p>
              </header>

              <DeviceList householdId={resolvedHousehold} />
            </section>
          )}

          <HouseholdDangerZone
            householdId={resolvedHousehold}
            currentUserIsOwner={currentUserIsOwner}
            hasOtherMembers={hasOtherMembers}
          />
        </>
      )}
    </main>
  );
}

function HouseholdSettingsSkeleton() {
  return (
    <>
      <section className="household-settings-page__section">
        <header className="household-settings-page__section-header">
          <h2>General</h2>
          <Skeleton width="13rem" />
        </header>

        <div className="household-settings-page__form">
          <div className="household-settings-page__field">
            <Skeleton width="7rem" height="0.85rem" />
            <Skeleton height={40} borderRadius="var(--radius-md)" />
          </div>

          <Skeleton width={70} height={40} borderRadius="var(--radius-md)" />
        </div>
      </section>

      <section className="household-settings-page__section">
        <header className="household-settings-page__section-header">
          <h2>Members</h2>
          <Skeleton width="15rem" />
        </header>

        <div className="household-settings-page__members">
          {Array.from({ length: 2 }).map((_, index) => (
            <div key={index} className="household-settings-page__member">
              <div className="household-settings-page__member-info">
                <Skeleton width="9rem" />
                <Skeleton
                  width="3.5rem"
                  height="1.25rem"
                  borderRadius="var(--radius-sm)"
                />
              </div>
            </div>
          ))}
        </div>
      </section>
    </>
  );
}
