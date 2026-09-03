import { useAuth } from "../context/AuthContext";
import { LogOut } from "lucide-react";
import "./LogoutButton.css";

export function LogoutButton() {
  const { logout } = useAuth();

  function handleLogout() {
    void logout();
  }

  return (
    <button
      type="button"
      className="button button--ghost logout-button"
      onClick={handleLogout}
    >
      <LogOut className="logout-button__icon" />
      <span>Logout</span>
    </button>
  );
}
