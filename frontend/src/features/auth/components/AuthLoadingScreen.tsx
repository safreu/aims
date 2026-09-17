import Skeleton from "react-loading-skeleton";

import styles from "./AuthLoadingScreen.module.css";

export function AuthLoadingScreen() {
  return (
    <main
      className={styles.container}
      aria-busy="true"
      aria-label="Loading application"
    >
      <div className={styles.content}>
        <Skeleton
          width="7rem"
          height="1.75rem"
          borderRadius="var(--radius-sm)"
        />

        <div className={styles.card}>
          <Skeleton
            width="55%"
            height="1.5rem"
            borderRadius="var(--radius-sm)"
          />

          <Skeleton
            count={3}
            height="var(--control-height)"
            borderRadius="var(--radius-sm)"
          />
        </div>
      </div>
    </main>
  );
}
