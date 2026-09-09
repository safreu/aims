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
import { UserEventsLayout } from "./features/households/events/UserEventsLayout";
import { AccountsPage } from "./pages/accounts/AccountsPage";
import { SettingsProviderLayout } from "./features/accounts/components/layout/SettingsProviderLayout";
import { ScannerPage } from "./pages/scanning/ScannerPage";
import { QueryClientProvider } from "@tanstack/react-query";
import { queryClient } from "./api/queryClient";
import { SkeletonTheme } from "react-loading-skeleton";
import "react-loading-skeleton/dist/skeleton.css";

function App() {
  return (
    <SkeletonTheme baseColor="var(--gray-4)" highlightColor="var(--gray-6)">
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <Routes>
            <Route element={<RequireGuest />}>
              <Route path="/login" element={<LoginPage />} />
              <Route path="/register" element={<RegisterPage />} />
            </Route>

            <Route element={<RequireAuth />}>
              <Route element={<SettingsProviderLayout />}>
                <Route element={<UserEventsLayout />}>
                  <Route path="/households" element={<HouseholdsPage />} />
                  <Route path="/account" element={<AccountsPage />} />

                  <Route
                    path="/households/:householdId"
                    element={<HouseholdLayout />}
                  >
                    <Route
                      index
                      element={<Navigate to="inventory" replace />}
                    />
                    <Route path="inventory" element={<InventoryPage />} />
                    <Route path="shopping" element={<ShoppingPage />} />
                    <Route
                      path="settings"
                      element={<HouseholdSettingsPage />}
                    />
                    <Route path="scanner" element={<ScannerPage />} />
                  </Route>
                </Route>
              </Route>
            </Route>
          </Routes>
        </BrowserRouter>
      </QueryClientProvider>
    </SkeletonTheme>
  );
}

export default App;
