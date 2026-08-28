import { Link, Outlet, useParams } from "react-router-dom";

export function HouseholdLayout() {
  const { householdId } = useParams();

  if (householdId === undefined) {
    throw new Error("HouseholdLayout requires a HouseholdId");
  }

  return (
    <>
      <header>
        <Link to={`/households`}>Households</Link>
        <nav>
          <Link to={`/households/${householdId}/inventory`}>Inventory</Link>
          {" | "}
          <Link to={`/households/${householdId}/shopping`}>shopping</Link>
        </nav>
      </header>

      <Outlet />
    </>
  );
}
