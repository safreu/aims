import { AppHeader } from "../../components/layout/AppHeader";
import { exportDiagnostics } from "../../diagnostics/exportDiagnostics";
import { useTrackingMode } from "../../features/accounts/components/tracking-mode/TrackingModeContext";
import { useAuth } from "../../features/auth/context/AuthContext";
import "./AccountsPage.css";

export function AccountsPage() {
  const { user } = useAuth();
  const { trackingMode, setTrackingMode } = useTrackingMode();

  if (user === null) {
    return null;
  }

  return (
    <>
      <AppHeader />

      <div className="account-page">
        <header className="account-page__header">
          <h1>Account</h1>
          <p>Manage your personal account settings</p>
        </header>

        <section className="account-section">
          <div className="account-profile">
            <div className="account-profile__avatar">
              {user.display_name.charAt(0).toUpperCase()}
            </div>

            <div className="account-profile__identity">
              <h2>{user.display_name}</h2>
              <p>{user.email}</p>
            </div>
          </div>

          <div className="account-section__content">
            <div className="account-field">
              <div>
                <span className="account-field__label">Display Name</span>
                <p className="account-field__description">
                  The name shown to other households
                </p>
              </div>

              <span className="account-field__value">{user.display_name}</span>
            </div>

            <div className="account-field">
              <div>
                <span className="account-field__label">Email</span>

                <p className="account-field__description">
                  Used to sign into your account
                </p>
              </div>

              <span className="account-field__value">{user.email}</span>
            </div>
          </div>
        </section>

        <section className="account-section">
          <div className="account-field">
            <div>
              <span className="account-field__label">Inventory tracking</span>
              <p className="account-field__description">
                Choose how you normally update inventory stock
              </p>
            </div>

            <div className="account-tracking-mode">
              <button
                type="button"
                className={`account-tracking-mode__option ${
                  trackingMode === "qr"
                    ? "account-tracking-mode__option--active"
                    : ""
                }`}
                onClick={() => setTrackingMode("qr")}
              >
                QR
              </button>

              <button
                type="button"
                className={`account-tracking-mode__option ${
                  trackingMode === "manual"
                    ? "account-tracking-mode__option--active"
                    : ""
                }`}
                onClick={() => setTrackingMode("manual")}
              >
                Manual
              </button>
            </div>
          </div>
        </section>

        <section className="account-section">
          <div className="account-field">
            <div>
              <span className="account-field__label">Diagnostics</span>
              <p className="account-field__description">
                Export recent technical logs to help diagnose problems with Aims
              </p>
            </div>

            <div className="account-diagnostics-actions">
              <button
                type="button"
                className="account-action-button"
                onClick={exportDiagnostics}
              >
                Export Diagnostics
              </button>
            </div>
          </div>
        </section>
      </div>
    </>
  );
}
