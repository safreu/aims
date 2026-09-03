import { useState, type SubmitEvent } from "react";

import { register } from "../../features/auth/api";
import { ApiError } from "../../api/client";
import { Link, useNavigate } from "react-router-dom";
import { useToast } from "../../components/toast/ToastContext";

export function RegisterPage() {
  const navigate = useNavigate();
  const { showToast } = useToast();

  const [email, setEmail] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [password, setPassword] = useState("");

  const [validationError, setValidationError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  function handleSubmit(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    setValidationError(null);
    setIsSubmitting(true);

    void register({
      email,
      display_name: displayName,
      password,
    })
      .then(() => {
        showToast("Account created", "success");
        navigate("/login", { replace: true });
      })
      .catch((error) => {
        if (error instanceof ApiError && error.status === 409) {
          setValidationError("An account with this email already exists");
          return;
        }

        showToast("Registration failed", "error");
      })
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
            <h2>Create an account</h2>
            <p>Get started with Aims</p>
          </header>

          <form className="auth-form" onSubmit={handleSubmit}>
            <label className="auth-form__field">
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
                autoComplete="new-password"
                required
                disabled={isSubmitting}
              />
            </label>

            {validationError !== null && (
              <p className="auth-form_error">{validationError}</p>
            )}

            <button
              type="submit"
              className="button button--primary auth-form__submit"
              disabled={isSubmitting}
            >
              {isSubmitting ? "Creating account..." : "Create account"}
            </button>
          </form>

          <p className="auth-card__switch">
            Already have an account? <Link to="/login">Sign in</Link>
          </p>
        </section>
      </div>
    </main>
  );
}
