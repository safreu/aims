import { useState, type SubmitEvent } from "react";

import { useToast } from "../../../../components/toast/ToastContext";
import { addHouseholdMember, removeHouseholdMember } from "../../api";
import type { Household, HouseholdMember } from "../../types";

import styles from "../../pages/HouseholdSettingsPage.module.css";

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
  const [removingMemberId, setRemovingMemberId] = useState<string | null>(null);
  const [isAdding, setIsAdding] = useState(false);

  async function handleRemoveMember(memberId: string) {
    setRemovingMemberId(memberId);

    try {
      await removeHouseholdMember(householdId, memberId);

      await onChanged();

      showToast("Household member removed", "success");
    } catch {
      showToast("Failed to remove household member", "error");
    } finally {
      setRemovingMemberId(null);
    }
  }

  async function handleAddMember(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    const email = memberEmail.trim();

    if (email === "") return;

    setIsAdding(true);

    try {
      await addHouseholdMember(householdId, {
        email,
      });

      await onChanged();

      setMemberEmail("");

      showToast("Household member added", "success");
    } catch {
      showToast("Failed to add household member", "error");
    } finally {
      setIsAdding(false);
    }
  }

  return (
    <section className={styles.section}>
      <header className={styles.sectionHeader}>
        <h2>Members</h2>
        <p>Manage who has access to this household</p>
      </header>

      <div className={styles.members}>
        {members.map((member) => {
          const isRemovingThisMember = removingMemberId === member.user_id;

          return (
            <div className={styles.member} key={member.user_id}>
              <div className={styles.memberInfo}>
                <span className={styles.memberName}>{member.display_name}</span>

                <span className={styles.memberRole}>{member.role}</span>
              </div>

              {currentUserIsOwner && member.role !== "owner" && (
                <button
                  type="button"
                  className="button button--ghost"
                  onClick={() => void handleRemoveMember(member.user_id)}
                  disabled={removingMemberId !== null}
                >
                  {isRemovingThisMember ? "Removing..." : "Remove"}
                </button>
              )}
            </div>
          );
        })}
      </div>

      {currentUserIsOwner && householdKind === "shared" && (
        <form
          className={`${styles.form} ${styles.addMember}`}
          onSubmit={handleAddMember}
        >
          <label className={styles.field}>
            <span>Add member</span>

            <input
              type="email"
              value={memberEmail}
              onChange={(event) => setMemberEmail(event.target.value)}
              placeholder="Email address"
              autoComplete="email"
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
