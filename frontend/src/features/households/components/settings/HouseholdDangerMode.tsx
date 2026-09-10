import { useNavigate } from "react-router-dom";
import { useToast } from "../../../../components/toast/ToastContext";
import { deleteHousehold, leaveHousehold } from "../../api";
import { ConfirmDialog } from "../../../../components/dialogs/ConfirmDialog";
import { useState } from "react";

type Props = {
  householdId: string;
  currentUserIsOwner: boolean;
  hasOtherMembers: boolean;
};

export function HouseholdDangerZone({
  householdId,
  currentUserIsOwner,
  hasOtherMembers,
}: Props) {
  const { showToast } = useToast();
  const navigate = useNavigate();

  const [isLeaving, setIsLeaving] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);

  const [confirmDeleteOpen, setConfirmDeleteOpen] = useState(false);
  const [confirmLeaveOpen, setConfirmLeaveOpen] = useState(false);

  async function handleDeleteHousehold() {
    setIsDeleting(true);

    try {
      await deleteHousehold(householdId);
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
      await leaveHousehold(householdId);
    } catch {
      showToast("Failed to leave household", "error");
      setIsLeaving(false);
      return;
    }

    showToast("You left this household", "success");
    navigate("/households");
  }

  return (
    <>
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
    </>
  );
}
