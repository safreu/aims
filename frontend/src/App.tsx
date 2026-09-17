import { lazy, Suspense } from "react";
import { QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import { SkeletonTheme } from "react-loading-skeleton";

import "react-loading-skeleton/dist/skeleton.css";

import { queryClient } from "./api/queryClient";
import { AppErrorBoundary } from "./components/error/AppErrorBoundary";
import { RouteLoadingScreen } from "./components/loading/RouteLoadingScreen";
import { SettingsProviderLayout } from "./features/accounts/components/layout/SettingsProviderLayout";
import { RequireAuth } from "./features/auth/guards/RequireAuth";
import { RequireGuest } from "./features/auth/guards/RequireGuest";
import { UserEventsLayout } from "./features/households/events/UserEventsLayout";
import { HouseholdLayout } from "./features/households/layouts/HouseholdLayout";

const LoginPage = lazy(() =>
  import("./features/auth/pages/LoginPage").then((module) => ({
    default: module.LoginPage,
  })),
);

const RegisterPage = lazy(() =>
  import("./features/auth/pages/RegisterPage").then((module) => ({
    default: module.RegisterPage,
  })),
);

const InventoryPage = lazy(() =>
  import("./features/inventory/pages/InventoryPage").then((module) => ({
    default: module.InventoryPage,
  })),
);

const ShoppingPage = lazy(() =>
  import("./features/shopping/pages/ShoppingPage").then((module) => ({
    default: module.ShoppingPage,
  })),
);

const HouseholdsPage = lazy(() =>
  import("./features/households/pages/HouseholdsPage").then((module) => ({
    default: module.HouseholdsPage,
  })),
);

const HouseholdSettingsPage = lazy(() =>
  import("./features/households/pages/HouseholdSettingsPage").then(
    (module) => ({
      default: module.HouseholdSettingsPage,
    }),
  ),
);

const AccountsPage = lazy(() =>
  import("./features/accounts/pages/AccountsPage").then((module) => ({
    default: module.AccountsPage,
  })),
);

const ScannerPage = lazy(() =>
  import("./features/scanning/pages/ScannerPage").then((module) => ({
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
            <Suspense fallback={<RouteLoadingScreen />}>
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
