import Skeleton from "react-loading-skeleton";

import styles from "./RouteLoadingScreen.module.css";

export function RouteLoadingScreen() {
  return (
    <main className={styles.page} aria-busy="true" aria-label="Loading page">
      <div className={styles.content}>
        <Skeleton
          width="10rem"
          height="1.75rem"
          borderRadius="var(--radius-sm)"
        />

        <Skeleton
          width="16rem"
          height="0.9rem"
          borderRadius="var(--radius-sm)"
        />

        <div className={styles.card}>
          <Skeleton
            height="var(--control-height)"
            borderRadius="var(--radius-sm)"
          />

          <Skeleton
            height="var(--control-height)"
            borderRadius="var(--radius-sm)"
          />

          <Skeleton
            height="var(--control-height)"
            borderRadius="var(--radius-sm)"
          />
        </div>
      </div>
    </main>
  );
}
