import { useState, type SubmitEvent } from "react";
import { Link, useNavigate } from "react-router-dom";

import { useToast } from "../../../components/toast/ToastContext";
import { login } from "../api";
import { useAuth } from "../context/AuthContext";

import styles from "./AuthPage.module.css";

export function LoginPage() {
  const navigate = useNavigate();
  const { refreshUser } = useAuth();
  const { showToast } = useToast();

  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);

  async function handleSubmit(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();
    setIsSubmitting(true);

    try {
      await login({
        email: email.trim(),
        password,
      });

      await refreshUser();

      navigate("/households", {
        replace: true,
      });
    } catch {
      showToast("Login failed", "error");
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
            <h2>Welcome back</h2>
            <p>Sign in to continue to Aims</p>
          </header>

          <form className={styles.form} onSubmit={handleSubmit}>
            <label className={styles.field}>
              <span>Email</span>

              <input
                type="email"
                value={email}
                onChange={(event) => setEmail(event.target.value)}
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
                autoComplete="current-password"
                required
                disabled={isSubmitting}
              />
            </label>

            <button
              type="submit"
              className={`button button--primary ${styles.submit}`}
              disabled={isSubmitting}
            >
              {isSubmitting ? "Signing in..." : "Login"}
            </button>
          </form>

          <p className={styles.switch}>
            Don't have an account? <Link to="/register">Create an account</Link>
          </p>
        </section>
      </div>
    </main>
  );
}
