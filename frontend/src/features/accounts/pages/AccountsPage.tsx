import { AppHeader } from "../../../components/layout/AppHeader";
import { exportDiagnostics } from "../../../diagnostics/exportDiagnostics";
import { useTrackingMode } from "../components/tracking-mode/TrackingModeContext";
import { useAuth } from "../../auth/context/AuthContext";

import styles from "./AccountsPage.module.css";

export function AccountsPage() {
  const { user } = useAuth();
  const { trackingMode, setTrackingMode } = useTrackingMode();

  if (user === null) {
    return null;
  }

  return (
    <>
      <AppHeader />

      <main className={styles.page}>
        <header className={styles.header}>
          <h1>Account</h1>
          <p>Manage your personal account settings</p>
        </header>

        <section className={styles.section}>
          <div className={styles.profile}>
            <div className={styles.avatar} aria-hidden="true">
              {user.display_name.charAt(0).toUpperCase()}
            </div>

            <div className={styles.identity}>
              <h2>{user.display_name}</h2>
              <p>{user.email}</p>
            </div>
          </div>

          <div className={styles.sectionContent}>
            <div className={styles.field}>
              <div>
                <span className={styles.fieldLabel}>Display name</span>

                <p className={styles.fieldDescription}>
                  The name shown to other households
                </p>
              </div>

              <span className={styles.fieldValue}>{user.display_name}</span>
            </div>

            <div className={styles.field}>
              <div>
                <span className={styles.fieldLabel}>Email</span>

                <p className={styles.fieldDescription}>
                  Used to sign into your account
                </p>
              </div>

              <span className={styles.fieldValue}>{user.email}</span>
            </div>
          </div>
        </section>

        <section className={styles.section}>
          <div className={styles.field}>
            <div>
              <span className={styles.fieldLabel}>Inventory tracking</span>

              <p className={styles.fieldDescription}>
                Choose how you normally update inventory stock
              </p>
            </div>

            <div
              className={styles.trackingMode}
              role="group"
              aria-label="Inventory tracking mode"
            >
              <button
                type="button"
                className={`${styles.trackingModeOption} ${
                  trackingMode === "qr" ? styles.trackingModeOptionActive : ""
                }`}
                aria-pressed={trackingMode === "qr"}
                onClick={() => setTrackingMode("qr")}
              >
                QR
              </button>

              <button
                type="button"
                className={`${styles.trackingModeOption} ${
                  trackingMode === "manual"
                    ? styles.trackingModeOptionActive
                    : ""
                }`}
                aria-pressed={trackingMode === "manual"}
                onClick={() => setTrackingMode("manual")}
              >
                Manual
              </button>
            </div>
          </div>
        </section>

        <section className={styles.section}>
          <div className={styles.field}>
            <div>
              <span className={styles.fieldLabel}>Diagnostics</span>

              <p className={styles.fieldDescription}>
                Export recent technical logs to help diagnose problems with Aims
              </p>
            </div>

            <div className={styles.diagnosticsActions}>
              <button
                type="button"
                className="button button--secondary"
                onClick={exportDiagnostics}
              >
                Export diagnostics
              </button>
            </div>
          </div>
        </section>
      </main>
    </>
  );
}
