import { useState, type SubmitEvent } from "react";

import { register } from "../features/auth/api";
import { ApiError } from "../api/client";
import { useNavigate } from "react-router-dom";

export function RegisterPage() {
  const navigate = useNavigate();
  const [email, setEmail] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);

  function handleSubmit(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);

    void register({
      email,
      display_name: displayName,
      password,
    })
      .then(() => {
        navigate("/login", { replace: true });
      })
      .catch((error) => {
        if (error instanceof ApiError && error.status === 409) {
          setError("An account with this email already exists");
          return;
        }
        setError("Registration failed");
      });
  }

  return (
    <main>
      <h1>Register</h1>

      <form onSubmit={handleSubmit}>
        <label>
          Name
          <input
            value={displayName}
            onChange={(event) => setDisplayName(event.target.value)}
          />
        </label>

        <label>
          Email
          <input
            type="email"
            value={email}
            onChange={(event) => setEmail(event.target.value)}
          />
        </label>

        <label>
          Password
          <input
            type="password"
            value={password}
            onChange={(event) => setPassword(event.target.value)}
          />
        </label>

        <button type="submit">Register</button>
      </form>
      {error && <p>{error}</p>}
    </main>
  );
}
