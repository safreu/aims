import { useAuth } from "./AuthProvider";

export function LogoutButton() {
  const { logout } = useAuth();

  function handleLogout() {
    void logout();
  }

  return (
    <button type="button" onClick={handleLogout}>
      Logout
    </button>
  );
}
