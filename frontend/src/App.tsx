<<<<<<< HEAD
import { BrowserRouter, Link, Route, Routes } from "react-router-dom";
import "./App.css";
import { LoginPage } from "./pages/LoginPage";
import { RegisterPage } from "./pages/RegisterPage";
import { InventoryPage } from "./pages/InventoryPage";
import { ShoppingPage } from "./pages/ShoppingPage";
import { RequireAuth } from "./features/auth/RequireAuth";
import { LogoutButton } from "./features/auth/LogoutButton";
import { RequireGuest } from "./features/auth/RequireGuest";

function App() {
=======
import { useState } from "react";
import reactLogo from "./assets/react.svg";
import viteLogo from "./assets/vite.svg";
import heroImg from "./assets/hero.png";
import "./App.css";

function App() {
  const [count, setCount] = useState(0);

>>>>>>> ff276be (feat: Implementing frontend ci, also frontend gets only run on frontend and backend only on backend changes)
  return (
    <BrowserRouter>
      <nav>
        <Link to="/login">Login</Link>
        {" | "}
        <Link to="/register">Register</Link>
        {" | "}
        <Link to="/inventory">Inventory</Link>
        {" | "}
        <Link to="/shopping">Shopping</Link>
      </nav>

      <LogoutButton />

      <Routes>
        <Route element={<RequireGuest />}>
          <Route path="/login" element={<LoginPage />} />
          <Route path="/register" element={<RegisterPage />} />
        </Route>

        <Route element={<RequireAuth />}>
          <Route path="/inventory" element={<InventoryPage />} />
          <Route path="/shopping" element={<ShoppingPage />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
