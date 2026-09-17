import { useCallback, useEffect } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import Skeleton from "react-loading-skeleton";
import { useParams } from "react-router-dom";

import { queryKeys } from "../../../api/queryKeys";
import { useTrackingMode } from "../../accounts/components/tracking-mode/TrackingModeContext";
import { useAuth } from "../../auth/context/AuthContext";
import { DeviceList } from "../../devices/components/DeviceList";
import { getHousehold, getHouseholdMembers } from "../api";
import { GeneralHouseholdSettings } from "../components/settings/GeneralHouseholdSettings";
import { HouseholdDangerZone } from "../components/settings/HouseholdDangerZone";
import { HouseholdMembersSettings } from "../components/settings/HouseholdMembersSettings";
import { useHouseholdEvents } from "../events/HouseholdEventsContext";

import styles from "./HouseholdSettingsPage.module.css";

export function HouseholdSettingsPage() {
  const { householdId } = useParams();
  const { user } = useAuth();
  const { subscribe } = useHouseholdEvents();
  const { trackingMode } = useTrackingMode();

  if (householdId === undefined) {
    throw new Error("HouseholdSettingsPage requires a householdId");
  }

  const resolvedHouseholdId = householdId;
  const queryClient = useQueryClient();

  const {
    data: household,
    isPending: isHouseholdPending,
    isError: isHouseholdError,
  } = useQuery({
    queryKey: queryKeys.household.detail(resolvedHouseholdId),
    queryFn: () => getHousehold(resolvedHouseholdId),
  });

  const {
    data: members = [],
    isPending: areMembersPending,
    isError: areMembersError,
  } = useQuery({
    queryKey: queryKeys.household.members(resolvedHouseholdId),
    queryFn: () => getHouseholdMembers(resolvedHouseholdId),
  });

  const currentMember = members.find((member) => member.user_id === user?.id);

  const currentUserIsOwner = currentMember?.role === "owner";

  const hasOtherMembers = members.some((member) => member.user_id !== user?.id);

  const refreshHouseholdSettings = useCallback(() => {
    return queryClient.invalidateQueries({
      queryKey: queryKeys.household.detail(resolvedHouseholdId),
    });
  }, [queryClient, resolvedHouseholdId]);

  const refreshMembers = useCallback(() => {
    return queryClient.invalidateQueries({
      queryKey: queryKeys.household.members(resolvedHouseholdId),
    });
  }, [queryClient, resolvedHouseholdId]);

  useEffect(() => {
    return subscribe("household_changed", () => {
      void refreshHouseholdSettings();
    });
  }, [subscribe, refreshHouseholdSettings]);

  const isPending = isHouseholdPending || areMembersPending;

  const isError = isHouseholdError || areMembersError;

  return (
    <main className={styles.page}>
      <header className={styles.header}>
        <h1>Household settings</h1>

        <p>
          {currentUserIsOwner
            ? "Manage this household and its members"
            : "View this household and manage your access"}
        </p>
      </header>

      {isPending ? (
        <HouseholdSettingsSkeleton />
      ) : isError || household === undefined ? (
        <p className={styles.status}>Failed to load household settings</p>
      ) : (
        <>
          <GeneralHouseholdSettings
            householdId={resolvedHouseholdId}
            householdName={household.name}
            currentUserIsOwner={currentUserIsOwner}
            onChanged={refreshHouseholdSettings}
          />

          <HouseholdMembersSettings
            householdId={resolvedHouseholdId}
            householdKind={household.kind}
            members={members}
            currentUserIsOwner={currentUserIsOwner}
            onChanged={refreshMembers}
          />

          {trackingMode === "qr" && (
            <section className={styles.section}>
              <header className={styles.sectionHeader}>
                <h2>Devices</h2>
                <p>Manage devices connected to this household</p>
              </header>

              <DeviceList householdId={resolvedHouseholdId} />
            </section>
          )}

          <HouseholdDangerZone
            householdId={resolvedHouseholdId}
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
      <section className={styles.section}>
        <header className={styles.sectionHeader}>
          <h2>General</h2>
          <Skeleton width="13rem" />
        </header>

        <div className={styles.form}>
          <div className={styles.field}>
            <Skeleton width="7rem" height="0.85rem" />

            <Skeleton
              height="var(--control-height)"
              borderRadius="var(--radius-sm)"
            />
          </div>

          <Skeleton
            width={70}
            height="var(--control-height)"
            borderRadius="var(--radius-sm)"
          />
        </div>
      </section>

      <section className={styles.section}>
        <header className={styles.sectionHeader}>
          <h2>Members</h2>
          <Skeleton width="15rem" />
        </header>

        <div className={styles.members}>
          {Array.from({ length: 2 }).map((_, index) => (
            <div key={index} className={styles.member}>
              <div className={styles.memberInfo}>
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
