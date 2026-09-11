import Skeleton from "react-loading-skeleton";
import "./AuthLoadingScreen.css";
export function AuthLoadingScreen() {
  return (
    <main className="auth-loading">
      <div className="auth-loading__content">
        <Skeleton
          width="7rem"
          height="1.75rem"
          borderRadius="var(--radius-sm)"
        />

        <div className="auth-loading__card">
          <Skeleton
            width="55%"
            height="1.5rem"
            borderRadius="var(--radius-sm)"
          />

          <Skeleton count={3} height={44} borderRadius="var(--radius-sm)" />
        </div>
      </div>
    </main>
  );
}
