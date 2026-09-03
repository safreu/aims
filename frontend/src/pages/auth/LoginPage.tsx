import { useState, type SubmitEvent } from "react";

import { login } from "../../features/auth/api";
import { useAuth } from "../../features/auth/context/AuthContext";
import { Link, useNavigate } from "react-router-dom";
import { useToast } from "../../components/toast/ToastContext";

import "./AuthPage.css";

export function LoginPage() {
  const navigate = useNavigate();
  const { refreshUser } = useAuth();
  const { showToast } = useToast();

  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);

  function handleSubmit(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();
    setIsSubmitting(true);

    void login({ email, password })
      .then(async () => {
        await refreshUser();
        navigate("/households", { replace: true });
      })
      .catch(() => showToast("Login failed", "error"))
      .finally(() => setIsSubmitting(false));
  }

  return (
    <main className="auth-page">
      <div className="auth-page__container">
        <header className="auth-page__brand">
          <h1>Aims</h1>
        </header>

        <section className="auth-card">
          <header className="auth-card__header">
            <h1>Welcome back</h1>
            <p>Sign in to continue to Aims</p>
          </header>

          <form className="auth-form" onSubmit={handleSubmit}>
            <label className="auth-form__field">
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

            <label className="auth-form__field">
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
              className="button button--primary auth-form__submit"
              disabled={isSubmitting}
            >
              {isSubmitting ? "Signing in..." : "Login"}
            </button>
          </form>

          <p className="auth-card__switch">
            Don't have an account? <Link to="/register">Create an account</Link>
          </p>
        </section>
      </div>
    </main>
  );
}
