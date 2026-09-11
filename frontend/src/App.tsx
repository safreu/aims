import { lazy, Suspense } from "react";
import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import "./App.css";
import { RequireAuth } from "./features/auth/guards/RequireAuth";
import { RequireGuest } from "./features/auth/guards/RequireGuest";
import { HouseholdLayout } from "./features/households/layouts/HouseholdLayout";
import { UserEventsLayout } from "./features/households/events/UserEventsLayout";
import { SettingsProviderLayout } from "./features/accounts/components/layout/SettingsProviderLayout";
import { QueryClientProvider } from "@tanstack/react-query";
import { queryClient } from "./api/queryClient";
import { SkeletonTheme } from "react-loading-skeleton";
import "react-loading-skeleton/dist/skeleton.css";
import { AppErrorBoundary } from "./components/error/AppErrorBoundary";

const LoginPage = lazy(() =>
  import("./pages/auth/LoginPage").then((module) => ({
    default: module.LoginPage,
  })),
);

const RegisterPage = lazy(() =>
  import("./pages/auth/RegisterPage").then((module) => ({
    default: module.RegisterPage,
  })),
);

const InventoryPage = lazy(() =>
  import("./pages/inventory/InventoryPage").then((module) => ({
    default: module.InventoryPage,
  })),
);

const ShoppingPage = lazy(() =>
  import("./pages/shopping/ShoppingPage").then((module) => ({
    default: module.ShoppingPage,
  })),
);

const HouseholdsPage = lazy(() =>
  import("./pages/households/HouseholdsPage").then((module) => ({
    default: module.HouseholdsPage,
  })),
);

const HouseholdSettingsPage = lazy(() =>
  import("./pages/households/HouseholdSettingsPage").then((module) => ({
    default: module.HouseholdSettingsPage,
  })),
);

const AccountsPage = lazy(() =>
  import("./pages/accounts/AccountsPage").then((module) => ({
    default: module.AccountsPage,
  })),
);

const ScannerPage = lazy(() =>
  import("./pages/scanning/ScannerPage").then((module) => ({
    default: module.ScannerPage,
  })),
);

const NotFoundPage = lazy(() =>
  import("./pages/NotFoundPage").then((module) => ({
    default: module.NotFoundPage,
  })),
);

function App() {
  return (
    <AppErrorBoundary>
      <SkeletonTheme baseColor="var(--gray-4)" highlightColor="var(--gray-6)">
        <QueryClientProvider client={queryClient}>
          <BrowserRouter>
            <Suspense fallback={<div>Loading...</div>}>
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

                <Route path="*" element={<NotFoundPage />} />
              </Routes>
            </Suspense>
          </BrowserRouter>
        </QueryClientProvider>
      </SkeletonTheme>
    </AppErrorBoundary>
  );
}

export default App;
