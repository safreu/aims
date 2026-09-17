import "./diagnostics/setup";

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import "./index.css";

import App from "./App";
import { ToastProvider } from "./components/toast/ToastProvider";
import { AuthProvider } from "./features/auth/context/AuthProvider";

const rootElement = document.getElementById("root");

if (rootElement === null) {
  throw new Error("Root element with id 'root' was not found");
}

createRoot(rootElement).render(
  <StrictMode>
    <ToastProvider>
      <AuthProvider>
        <App />
      </AuthProvider>
    </ToastProvider>
  </StrictMode>,
);
