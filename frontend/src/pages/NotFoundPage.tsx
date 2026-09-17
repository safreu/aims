import { Link } from "react-router-dom";

import styles from "./NotFoundPage.module.css";

export function NotFoundPage() {
  return (
    <main className={styles.page}>
      <div className={styles.content}>
        <span className={styles.code} aria-hidden="true">
          404
        </span>

        <h1>Page not found</h1>

        <p>The page you were looking for does not exist.</p>

        <Link to="/households" className="button button--primary">
          Go to households
        </Link>
      </div>
    </main>
  );
}
