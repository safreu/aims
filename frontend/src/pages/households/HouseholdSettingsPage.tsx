import { useCallback, useEffect, useState, type SubmitEvent } from "react";
import "./HouseholdSettingsPage.css";
import type {
  Household,
  HouseholdMember,
} from "../../features/households/types";
import { useNavigate, useParams } from "react-router-dom";
import {
  addHouseholdMembers as addHouseholdMember,
  deleteHousehold,
  getHousehold,
  getHouseholdMembers,
  leaveHousehold,
  removeHouseholdMember,
  renameHousehold,
} from "../../features/households/api";
import { useToast } from "../../components/toast/ToastContext";
import { useAuth } from "../../features/auth/context/AuthContext";
import { ConfirmDialog } from "../../components/dialogs/ConfirmDialog";
import { isHouseholdAccessError } from "../../features/households/errors";
import { useHouseholdEvents } from "../../features/households/events/HouseholdEventsContext";
import { DeviceList } from "../../features/devices/components/DeviceList";

export function HouseholdSettingsPage() {
  const { householdId } = useParams();
  const { showToast } = useToast();
  const { user } = useAuth();
  const navigate = useNavigate();
  const { subscribe } = useHouseholdEvents();

  if (householdId === undefined) {
    throw new Error("HouseholdSettingsPage requires a householdId");
  }

  const resolvedHousehold = householdId;

  const [household, setHousehold] = useState<Household | null>(null);
  const [members, setMembers] = useState<HouseholdMember[]>([]);

  const [memberEmail, setMemberEmail] = useState("");

  const [loading, setLoading] = useState(true);
  const [isRenaming, setIsRenaming] = useState(false);
  const [isRemoving, setIsRemoving] = useState(false);
  const [isAdding, setIsAdding] = useState(false);
  const [isLeaving, setIsLeaving] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);

  const [confirmDeleteOpen, setConfirmDeleteOpen] = useState(false);
  const [confirmLeaveOpen, setConfirmLeaveOpen] = useState(false);

  const [name, setName] = useState("");

  const currentMember = members.find((member) => member.user_id == user?.id);

  const currentUserIsOwner = currentMember?.role === "owner";

  const hasOtherMembers = members.some((member) => member.user_id !== user?.id);

  const refreshHouseholdSettings = useCallback(async () => {
    const [household, members] = await Promise.all([
      getHousehold(resolvedHousehold),
      getHouseholdMembers(resolvedHousehold),
    ]);

    setHousehold(household);
    setName(household.name);
    setMembers(members);
  }, [resolvedHousehold]);

  async function refreshMembers() {
    const members = await getHouseholdMembers(resolvedHousehold);
    setMembers(members);
  }

  async function handleRename(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    if (name.trim() === "") return;

    setIsRenaming(true);

    await renameHousehold(resolvedHousehold, { name: name.trim() })
      .then(async () => {
        const household = await getHousehold(resolvedHousehold);
        setHousehold(household);
        setName(household.name);
      })
      .catch(() => showToast("Failed to rename household"))
      .finally(async () => setIsRenaming(false));
  }

  async function handleRemoveMember(memberId: string) {
    setIsRemoving(true);

    await removeHouseholdMember(resolvedHousehold, memberId)
      .then(async () => {
        await refreshMembers();
        showToast("Household member removed", "success");
      })
      .catch(() => showToast("Failed to remove household member", "error"))
      .finally(async () => setIsRemoving(false));
  }

  async function handleAddMember(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    if (memberEmail.trim() === "") return;

    setIsAdding(true);

    await addHouseholdMember(resolvedHousehold, { email: memberEmail.trim() })
      .then(async () => {
        await refreshMembers();
        setMemberEmail("");
        showToast("Household member added", "success");
      })
      .catch(() => showToast("Failed to add household member", "error"))
      .finally(async () => setIsAdding(false));
  }

  useEffect(() => {
    async function loadHouseholdSettings() {
      const [household, members] = await Promise.all([
        getHousehold(resolvedHousehold),
        getHouseholdMembers(resolvedHousehold),
      ]);

      setHousehold(household);
      setName(household.name);
      setMembers(members);
    }

    void loadHouseholdSettings()
      .catch((error) => {
        if (isHouseholdAccessError(error)) return;
        showToast("Failed to load household settings", "error");
      })
      .finally(() => setLoading(false));
  }, [resolvedHousehold, showToast]);

  useEffect(() => {
    return subscribe("household_changed", () => {
      void refreshHouseholdSettings().catch((error) => {
        if (isHouseholdAccessError(error)) return;
        showToast("Failed to refresh household settings", "error");
      });
    });
  }, [subscribe, showToast, refreshHouseholdSettings]);

  async function handleDeleteHousehold() {
    setIsDeleting(true);

    try {
      await deleteHousehold(resolvedHousehold);
    } catch {
      showToast("Failed to delete household", "error");
      setIsDeleting(false);
      return;
    }

    showToast("You deleted this household", "success");
    navigate("/households");
  }

  async function handleLeaveHousehold() {
    setIsLeaving(true);

    try {
      await leaveHousehold(resolvedHousehold);
    } catch {
      showToast("Failed to leave household", "error");
      setIsLeaving(false);
      return;
    }

    showToast("You left this household", "success");
    navigate("/households");
  }

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
                disabled={isRenaming}
              />
            </label>

            <button
              type="submit"
              className="button button--primary"
              disabled={isRenaming || name.trim() === ""}
            >
              {isRenaming ? "Saving..." : "Save"}
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
                  disabled={isRemoving}
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
                disabled={isAdding}
              />
            </label>

            <button
              type="submit"
              className="button button--primary"
              disabled={isAdding || memberEmail.trim() === ""}
            >
              {isAdding ? "Adding member..." : "Add member"}
            </button>
          </form>
        )}
      </section>

      <section className="household-settings-page__section">
        <header className="household-settings-page__section-header">
          <h2>Devices</h2>
          <p>Manage devices connected to this household</p>
        </header>

        <DeviceList householdId={resolvedHousehold} />
      </section>

      <section className="household-settings-page__section household-settings-page__danger-zone">
        <header className="household-settings-page__section-header">
          <h2>Danger zone</h2>
          <p>Actions that affect your access to this household</p>
        </header>

        {currentUserIsOwner ? (
          <div className="household-settings-page__danger-action">
            <div>
              <strong>Delete household</strong>
              <p>
                {hasOtherMembers
                  ? "Remove all other members before deleting this household"
                  : "Permanently delete this household and all of its data"}
              </p>
            </div>

            <button
              type="button"
              className="button button--danger"
              disabled={hasOtherMembers || isDeleting}
              onClick={() => setConfirmDeleteOpen(true)}
            >
              {isDeleting ? "Deleting..." : "Delete household"}
            </button>
          </div>
        ) : (
          <div className="household-settings-page__danger-action">
            <div>
              <strong>Leave household</strong>
              <p>You will lose access to this household</p>
            </div>

            <button
              type="button"
              className="button button--danger"
              disabled={isLeaving}
              onClick={() => setConfirmLeaveOpen(true)}
            >
              {isLeaving ? "Leaving..." : "Leave household"}
            </button>
          </div>
        )}
      </section>

      <ConfirmDialog
        open={confirmDeleteOpen}
        title="Delete household?"
        description="This permanently deletes this household and all of its data. This action cannot be undone!"
        confirmLabel="Delete household"
        destructive
        loading={isDeleting}
        onConfirm={handleDeleteHousehold}
        onCancel={() => setConfirmDeleteOpen(false)}
      />

      <ConfirmDialog
        open={confirmLeaveOpen}
        title="Leave household?"
        description="You will lose access to this household"
        confirmLabel="Leave household"
        destructive
        loading={isLeaving}
        onConfirm={handleLeaveHousehold}
        onCancel={() => setConfirmLeaveOpen(false)}
      />
    </main>
  );
}
