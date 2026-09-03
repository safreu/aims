import { useEffect, useState } from "react";
import type { Household } from "../../features/households/types";
import { getHouseholds } from "../../features/households/api";
import { Link } from "react-router-dom";
import { useToast } from "../../components/toast/ToastContext";
import { Plus, User, Users } from "lucide-react";
import { CreateHouseholdDialog } from "../../features/households/components/CreateHouseholdDialog";
import { AppHeader } from "../../components/layout/AppHeader";

export function HouseholdsPage() {
  const [households, setHouseholds] = useState<Household[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreateDialog, setShowCreateDialog] = useState(false);

  const { showToast } = useToast();

  async function refreshHouseholds() {
    const households = await getHouseholds();
    setHouseholds(households);
  }

  useEffect(() => {
    void getHouseholds()
      .then((households) => setHouseholds(households))
      .catch(() => showToast("Failed to load households", "error"))
      .finally(() => setLoading(false));
  }, [showToast]);

  if (loading) {
    return (
      <main className="households-page">
        <p className="households-page__status">Loading households...</p>
      </main>
    );
  }

  return (
    <>
      <AppHeader />
      <main className="households-page">
        <header className="households-page__header">
          <div>
            <h1>Households</h1>
            <p>Select a household above to continue.</p>
          </div>

          <button
            type="button"
            className="button button--primary"
            onClick={() => setShowCreateDialog(true)}
          >
            <Plus size={18} />
            Create household
          </button>
        </header>

        {households.length === 0 ? (
          <section className="households-page__empty">
            <Users className="households-page__empty-icon" />
            <h2>No households yet</h2>
            <p>Create your first household to start managing inventory</p>

            <button
              type="button"
              className="button button--primary"
              onClick={() => setShowCreateDialog(true)}
            >
              <Plus size={18} />
              Create household
            </button>
          </section>
        ) : (
          <section className="households-page__grid">
            {households.map((household) => (
              <Link
                key={household.id}
                to={`/households/${household.id}/inventory`}
                className="household-card"
              >
                <div className="household-card__icon">
                  {household.kind === "shared" ? <Users /> : <User />}
                </div>

                <div className="household-card__content">
                  <h2>{household.name}</h2>

                  <p>
                    {household.kind === "shared"
                      ? "Shared household"
                      : "Personal household"}
                  </p>
                </div>
              </Link>
            ))}
          </section>
        )}

        {showCreateDialog && (
          <CreateHouseholdDialog
            onCreated={refreshHouseholds}
            onClose={() => setShowCreateDialog(false)}
          />
        )}
      </main>
    </>
  );
}
