import { useState, type SubmitEvent } from "react";
import { useToast } from "../../../../components/toast/ToastContext";
import { renameHousehold } from "../../api";

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

  async function handleRename(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    const trimmedName = resolvedName.trim();

    if (trimmedName === "") return;

    setIsRenaming(true);

    await renameHousehold(householdId, { name: trimmedName })
      .then(async () => {
        await onChanged();
        setName(null);
      })
      .catch(() => showToast("Failed to rename household", "error"))
      .finally(async () => setIsRenaming(false));
  }

  return (
    <section className="household-settings-page__section">
      <header className="household-settings-page__section-header">
        <h2>General</h2>
        <p>Change the name of this household</p>
      </header>

      {currentUserIsOwner ? (
        <form className="household-settings-page__form" onSubmit={handleRename}>
          <label className="household-settings-page__field">
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
            disabled={isRenaming || resolvedName.trim() === ""}
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
            {householdName}
          </span>
        </div>
      )}
    </section>
  );
}
