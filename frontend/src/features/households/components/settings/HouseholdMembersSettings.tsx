import { useState, type SubmitEvent } from "react";
import { useToast } from "../../../../components/toast/ToastContext";
import type { Household, HouseholdMember } from "../../types";
import { addHouseholdMember, removeHouseholdMember } from "../../api";

type Props = {
  householdId: string;
  householdKind: Household["kind"];
  members: HouseholdMember[];
  currentUserIsOwner: boolean;
  onChanged: () => Promise<void>;
};

export function HouseholdMembersSettings({
  householdId,
  householdKind,
  members,
  currentUserIsOwner,
  onChanged,
}: Props) {
  const { showToast } = useToast();

  const [memberEmail, setMemberEmail] = useState("");
  const [isRemoving, setIsRemoving] = useState(false);
  const [isAdding, setIsAdding] = useState(false);

  async function handleRemoveMember(memberId: string) {
    setIsRemoving(true);

    await removeHouseholdMember(householdId, memberId)
      .then(async () => {
        await onChanged();
        showToast("Household member removed", "success");
      })
      .catch(() => showToast("Failed to remove household member", "error"))
      .finally(async () => setIsRemoving(false));
  }

  async function handleAddMember(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    const email = memberEmail.trim();

    if (email === "") return;

    setIsAdding(true);

    await addHouseholdMember(householdId, { email })
      .then(async () => {
        await onChanged();
        setMemberEmail("");
        showToast("Household member added", "success");
      })
      .catch(() => showToast("Failed to add household member", "error"))
      .finally(async () => setIsAdding(false));
  }

  return (
    <section className="household-settings-page__section">
      <header className="household-settings-page__section-header">
        <h2>Members</h2>
        <p>Manage who has access to this household</p>
      </header>

      <div className="household-settings-page__members">
        {members.map((member) => (
          <div className="household-settings-page__member" key={member.user_id}>
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

      {currentUserIsOwner && householdKind === "shared" && (
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
  );
}
