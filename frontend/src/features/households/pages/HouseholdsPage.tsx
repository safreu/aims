import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { Plus, User, Users } from "lucide-react";
import Skeleton from "react-loading-skeleton";

import { queryKeys } from "../../../api/queryKeys";
import { AppHeader } from "../../../components/layout/AppHeader";
import { CreateHouseholdDialog } from "../components/CreateHouseholdDialog";
import { getHouseholds } from "../api";

import styles from "./HouseholdsPage.module.css";

export function HouseholdsPage() {
  const queryClient = useQueryClient();

  const [showCreateDialog, setShowCreateDialog] = useState(false);

  const {
    data: households = [],
    isPending,
    isError,
  } = useQuery({
    queryKey: queryKeys.household.all(),
    queryFn: getHouseholds,
  });

  async function refreshHouseholds() {
    await queryClient.invalidateQueries({
      queryKey: queryKeys.household.all(),
    });
  }

  return (
    <>
      <AppHeader />

      <main className={styles.page}>
        <header className={styles.header}>
          <div>
            <h1>Households</h1>
            <p>Choose a household to manage its inventory and shopping list.</p>
          </div>

          <div className={styles.createButton}>
            <button
              type="button"
              className="button button--primary"
              onClick={() => setShowCreateDialog(true)}
            >
              <Plus size={18} aria-hidden="true" />
              Create household
            </button>
          </div>
        </header>

        {isPending ? (
          <HouseholdsSkeleton />
        ) : isError ? (
          <p className={styles.status}>Failed to load households.</p>
        ) : households.length === 0 ? (
          <section className={styles.empty}>
            <Users className={styles.emptyIcon} aria-hidden="true" />

            <h2>No households yet</h2>

            <p>
              Create your first household to start managing inventory and
              shopping.
            </p>

            <button
              type="button"
              className="button button--primary"
              onClick={() => setShowCreateDialog(true)}
            >
              <Plus size={18} aria-hidden="true" />
              Create household
            </button>
          </section>
        ) : (
          <section className={styles.grid} aria-label="Households">
            {households.map((household) => (
              <Link
                key={household.id}
                to={`/households/${household.id}/inventory`}
                className={styles.card}
              >
                <div className={styles.cardIcon} aria-hidden="true">
                  {household.kind === "shared" ? <Users /> : <User />}
                </div>

                <div className={styles.cardContent}>
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

function HouseholdsSkeleton() {
  return (
    <div
      className={styles.grid}
      aria-label="Loading households"
      aria-busy="true"
    >
      {Array.from({ length: 3 }).map((_, index) => (
        <div key={index} className={`${styles.card} ${styles.skeleton}`}>
          <div className={styles.skeletonIcon}>
            <Skeleton
              width="100%"
              height="100%"
              borderRadius="var(--radius-md)"
            />
          </div>

          <div className={styles.skeletonContent}>
            <Skeleton width="8rem" height="1rem" />

            <Skeleton width="6rem" height="0.8rem" />
          </div>
        </div>
      ))}
    </div>
  );
}
