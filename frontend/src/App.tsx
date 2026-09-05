import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import "./App.css";
import { LoginPage } from "./pages/auth/LoginPage";
import { RegisterPage } from "./pages/auth/RegisterPage";
import { InventoryPage } from "./pages/inventory/InventoryPage";
import { ShoppingPage } from "./pages/shopping/ShoppingPage";
import { RequireAuth } from "./features/auth/guards/RequireAuth";
import { RequireGuest } from "./features/auth/guards/RequireGuest";
import { HouseholdsPage } from "./pages/households/HouseholdsPage";
import { HouseholdLayout } from "./features/households/layouts/HouseholdLayout";
import { HouseholdSettingsPage } from "./pages/households/HouseholdSettingsPage";

function App() {
  return (
    <BrowserRouter>
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
            <Route path="settings" element={<HouseholdSettingsPage />} />
          </Route>
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
