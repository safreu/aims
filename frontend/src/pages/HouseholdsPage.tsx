import { useEffect, useState, type SubmitEvent } from "react";
import type { Household } from "../features/households/types";
import { createHousehold, getHouseholds } from "../features/households/api";
import { Link } from "react-router-dom";

export function HouseholdsPage() {
  const [households, setHouseholds] = useState<Household[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [name, setName] = useState("");
  const [kind, setKind] = useState<"personal" | "shared">("shared");
  const [createError, setCreateError] = useState<string | null>(null);

  function handleCreateHousehold(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();
    setCreateError(null);

    void createHousehold({ name, kind })
      .then(async () => {
        setName("");

        const households = await getHouseholds();
        setHouseholds(households);
      })
      .catch(() => setCreateError("Failed to create a household"));
  }

  useEffect(() => {
    void getHouseholds()
      .then((households) => setHouseholds(households))
      .catch(() => setError("Failed to load households"))
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return <p>Loading Households...</p>;
  }

  if (error !== null) {
    return <p>{error}</p>;
  }

  return (
    <main>
      <h1>Households</h1>

      <form onSubmit={handleCreateHousehold}>
        <div>
          <label htmlFor="household-name">Name</label>
          <input
            id="household-name"
            type="text"
            value={name}
            onChange={(event) => setName(event.target.value)}
            required
          />
        </div>

        <div>
          <label htmlFor="household-kind">Kind</label>
          <select
            id="household-kind"
            value={kind}
            onChange={(event) =>
              setKind(event.target.value as "personal" | "shared")
            }
            required
          >
            <option value="shared">Shared</option>
            <option value="personal">Personal</option>
          </select>
        </div>

        <button type="submit">Create Household</button>

        {createError !== null && <p>{createError}</p>}
      </form>

      {households.length === 0 ? (
        <p>You dont have any households yet :( </p>
      ) : (
        <ul>
          {households.map((households) => (
            <li key={households.id}>
              <Link to={`/households/${households.id}/inventory`}>
                Inventory
              </Link>
              {" | "}
              <Link to={`/households/${households.id}/shopping`}>Shopping</Link>
              <br />
              {households.name}
              <br />
              {households.kind}
            </li>
          ))}
        </ul>
      )}
    </main>
  );
}
