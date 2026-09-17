import { useState, type SubmitEvent } from "react";

import { useToast } from "../../../../components/toast/ToastContext";
import { renameHousehold } from "../../api";

import styles from "../../pages/HouseholdSettingsPage.module.css";

type Props = {
  householdId: string;
  householdName: string;
  currentUserIsOwner: boolean;
  onChanged: () => Promise<void>;
};

export function GeneralHouseholdSettings({
  householdId,
  householdName,
  currentUserIsOwner,
  onChanged,
}: Props) {
  const { showToast } = useToast();

  const [name, setName] = useState<string | null>(null);
  const [isRenaming, setIsRenaming] = useState(false);

  const resolvedName = name ?? householdName;
  const trimmedName = resolvedName.trim();

  const hasChanges = trimmedName !== "" && trimmedName !== householdName;

  async function handleRename(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    if (!hasChanges) return;

    setIsRenaming(true);

    try {
      await renameHousehold(householdId, {
        name: trimmedName,
      });

      await onChanged();
      setName(null);

      showToast("Household renamed", "success");
    } catch {
      showToast("Failed to rename household", "error");
    } finally {
      setIsRenaming(false);
    }
  }

  return (
    <section className={styles.section}>
      <header className={styles.sectionHeader}>
        <h2>General</h2>
        <p>Change the name of this household</p>
      </header>

      {currentUserIsOwner ? (
        <form className={styles.form} onSubmit={handleRename}>
          <label className={styles.field}>
            <span>Household name</span>

            <input
              value={resolvedName}
              onChange={(event) => setName(event.target.value)}
              disabled={isRenaming}
            />
          </label>

          <button
            type="submit"
            className="button button--primary"
            disabled={isRenaming || !hasChanges}
          >
            {isRenaming ? "Saving..." : "Save"}
          </button>
        </form>
      ) : (
        <div className={styles.readonlyField}>
          <span className={styles.readonlyLabel}>Household name</span>

          <span className={styles.readonlyValue}>{householdName}</span>
        </div>
      )}
    </section>
  );
}
