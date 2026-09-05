import { useEffect, useState, type SubmitEvent } from "react";
import "./HouseholdSettingsPage.css";
import type {
  Household,
  HouseholdMember,
} from "../../features/households/types";
import { useParams } from "react-router-dom";
import {
  addHouseholdMembers as addHouseholdMember,
  getHousehold,
  getHouseholdMembers,
  removeHouseholdMember,
  renameHousehold,
} from "../../features/households/api";
import { useToast } from "../../components/toast/ToastContext";
import { useAuth } from "../../features/auth/context/AuthContext";

export function HouseholdSettingsPage() {
  const { householdId } = useParams();
  const { showToast } = useToast();
  const { user } = useAuth();

  if (householdId === undefined) {
    throw new Error("HouseholdSettingsPage requires a householdId");
  }

  const resolvedHousehold = householdId;

  const [household, setHousehold] = useState<Household | null>(null);
  const [members, setMembers] = useState<HouseholdMember[]>([]);

  const [memberEmail, setMemberEmail] = useState("");

  const [loading, setLoading] = useState(true);
  const [isMutating, setIsMutating] = useState(false);

  const [name, setName] = useState("");

  const currentMember = members.find((member) => member.user_id == user?.id);

  const currentUserIsOwner = currentMember?.role === "owner";

  async function refreshMembers() {
    const members = await getHouseholdMembers(resolvedHousehold);
    setMembers(members);
  }

  async function handleRename(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    if (name.trim() === "") return;

    setIsMutating(true);

    await renameHousehold(resolvedHousehold, { name: name.trim() })
      .then(async () => {
        const household = await getHousehold(resolvedHousehold);
        setHousehold(household);
        setName(household.name);
      })
      .catch(() => showToast("Failed to rename household"))
      .finally(async () => setIsMutating(false));
  }

  async function handleRemoveMember(memberId: string) {
    setIsMutating(true);

    await removeHouseholdMember(resolvedHousehold, memberId)
      .then(async () => {
        await refreshMembers();
        showToast("Household member removed", "success");
      })
      .catch(() => showToast("Failed to remove household member", "error"))
      .finally(async () => setIsMutating(false));
  }

  async function handleAddMember(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    if (memberEmail.trim() === "") return;

    setIsMutating(true);

    await addHouseholdMember(resolvedHousehold, { email: memberEmail.trim() })
      .then(async () => {
        await refreshMembers();
        setMemberEmail("");
        showToast("Household member added", "success");
      })
      .catch(() => showToast("Failed to add household member", "error"))
      .finally(async () => setIsMutating(false));
  }

  useEffect(() => {
    void Promise.all([
      getHousehold(householdId),
      getHouseholdMembers(householdId),
    ])
      .then(([household, members]) => {
        setHousehold(household);
        setName(household.name);
        setMembers(members);
      })
      .catch(() => {
        showToast("Failed to load household settings", "error");
      })
      .finally(() => setLoading(false));
  }, [householdId, showToast]);

  if (loading) {
    return (
      <main className="household-settings-page">
        <p>Loading household settings...</p>
      </main>
    );
  }

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

      <section className="household-settings-page__section">
        <header className="household-settings-page__section-header">
          <h2>General</h2>
          <p>Change the name of this household</p>
        </header>

        {currentUserIsOwner ? (
          <form
            className="household-settings-page__form"
            onSubmit={handleRename}
          >
            <label className="household-settings-page__field">
              <span>Household name</span>

              <input
                value={name}
                onChange={(event) => setName(event.target.value)}
                disabled={isMutating}
              />
            </label>

            <button
              type="submit"
              className="button button--primary"
              disabled={isMutating || name.trim() === ""}
            >
              {isMutating ? "Saving..." : "Save"}
            </button>
          </form>
        ) : (
          <div className="household-settings-page__readonly-field">
            <span className="household-settings-page__readonly-label">
              Household name
            </span>
            <span className="household-settings-page__readonly-value">
              {household?.name}
            </span>
          </div>
        )}
      </section>

      <section className="household-settings-page__section">
        <header className="household-settings-page__section-header">
          <h2>Members</h2>
          <p>Manage who has access to this household</p>
        </header>

        <div className="household-settings-page__members">
          {members.map((member) => (
            <div
              className="household-settings-page__member"
              key={member.user_id}
            >
              <div className="household-settings-page__member-info">
                <span className="household-settings-page__member-name">
                  {member.display_name}
                </span>

                <span className="household-settings-page__member-role">
                  {member.role}
                </span>
              </div>

              {currentUserIsOwner && member.role !== "owner" && (
                <button
                  type="button"
                  className="button button--ghost"
                  onClick={() => handleRemoveMember(member.user_id)}
                  disabled={isMutating}
                >
                  Remove
                </button>
              )}
            </div>
          ))}
        </div>

        {currentUserIsOwner && household?.kind === "shared" && (
          <form
            className="household-settings-page__form household-settings-page__add-member"
            onSubmit={handleAddMember}
          >
            <label className="household-settings-page__field">
              <span>Add member</span>

              <input
                type="email"
                value={memberEmail}
                onChange={(event) => setMemberEmail(event.target.value)}
                placeholder="Email address"
                disabled={isMutating}
              />
            </label>

            <button
              type="submit"
              className="button button--primary"
              disabled={isMutating || memberEmail.trim() === ""}
            >
              {isMutating ? "Adding member..." : "Add member"}
            </button>
          </form>
        )}
      </section>
    </main>
  );
}
