import { useState, type SubmitEvent } from "react";
import { Link, useNavigate } from "react-router-dom";

import { ApiError } from "../../../api/client";
import { useToast } from "../../../components/toast/ToastContext";
import { register } from "../api";

import styles from "./AuthPage.module.css";

export function RegisterPage() {
  const navigate = useNavigate();
  const { showToast } = useToast();

  const [email, setEmail] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [password, setPassword] = useState("");

  const [validationError, setValidationError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  async function handleSubmit(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    setValidationError(null);
    setIsSubmitting(true);

    try {
      await register({
        email: email.trim(),
        display_name: displayName.trim(),
        password,
      });

      showToast("Account created", "success");

      navigate("/login", {
        replace: true,
      });
    } catch (error) {
      if (error instanceof ApiError && error.status === 409) {
        setValidationError("An account with this email already exists");
        return;
      }

      showToast("Registration failed", "error");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <main className={styles.page}>
      <div className={styles.container}>
        <header className={styles.brand}>
          <h1>Aims</h1>
        </header>

        <section className={styles.card}>
          <header className={styles.cardHeader}>
            <h2>Create an account</h2>
            <p>Get started with Aims</p>
          </header>

          <form className={styles.form} onSubmit={handleSubmit}>
            <label className={styles.field}>
              <span>Name</span>

              <input
                type="text"
                value={displayName}
                onChange={(event) => setDisplayName(event.target.value)}
                autoComplete="name"
                required
                disabled={isSubmitting}
              />
            </label>

            <label className={styles.field}>
              <span>Email</span>

              <input
                type="email"
                value={email}
                onChange={(event) => {
                  setEmail(event.target.value);

                  if (validationError !== null) {
                    setValidationError(null);
                  }
                }}
                autoComplete="email"
                required
                disabled={isSubmitting}
              />
            </label>

            <label className={styles.field}>
              <span>Password</span>

              <input
                type="password"
                value={password}
                onChange={(event) => setPassword(event.target.value)}
                autoComplete="new-password"
                required
                disabled={isSubmitting}
              />
            </label>

            {validationError !== null && (
              <p className={styles.error} role="alert">
                {validationError}
              </p>
            )}

            <button
              type="submit"
              className={`button button--primary ${styles.submit}`}
              disabled={isSubmitting}
            >
              {isSubmitting ? "Creating account..." : "Create account"}
            </button>
          </form>

          <p className={styles.switch}>
            Already have an account? <Link to="/login">Sign in</Link>
          </p>
        </section>
      </div>
    </main>
  );
}
