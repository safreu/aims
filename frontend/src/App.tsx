import { BrowserRouter, Link, Navigate, Route, Routes } from "react-router-dom";
import "./App.css";
import { LoginPage } from "./pages/LoginPage";
import { RegisterPage } from "./pages/RegisterPage";
import { InventoryPage } from "./pages/InventoryPage";
import { ShoppingPage } from "./pages/ShoppingPage";
import { RequireAuth } from "./features/auth/RequireAuth";
import { LogoutButton } from "./features/auth/LogoutButton";
import { RequireGuest } from "./features/auth/RequireGuest";
import { HouseholdsPage } from "./pages/HouseholdsPage";
import { HouseholdLayout } from "./features/households/HouseholdLayout";

function App() {
  return (
    <BrowserRouter>
      <nav>
        <Link to="/login">Login</Link>
        {" | "}
        <Link to="/register">Register</Link>
        {" | "}
        <Link to="/households">Households</Link>
      </nav>

      <LogoutButton />

      <Routes>
        <Route element={<RequireGuest />}>
          <Route path="/login" element={<LoginPage />} />
          <Route path="/register" element={<RegisterPage />} />
        </Route>

        <Route element={<RequireAuth />}>
          <Route path="/households" element={<HouseholdsPage />} />

          <Route path="/households/:householdId" element={<HouseholdLayout />}>
            <Route index element={<Navigate to="inventory" replace />} />
            <Route path="inventory" element={<InventoryPage />} />
            <Route path="shopping" element={<ShoppingPage />} />
          </Route>
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
