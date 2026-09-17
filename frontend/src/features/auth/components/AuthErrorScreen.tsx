import styles from "./AuthErrorScreen.module.css";

type Props = {
  onRetry: () => void;
};

export function AuthErrorScreen({ onRetry }: Props) {
  return (
    <main className={styles.container}>
      <div className={styles.content} role="alert">
        <h1 className={styles.title}>Unable to connect</h1>

        <p className={styles.description}>
          Aims could not determine your login status. Check your connection and
          try again.
        </p>

        <button
          type="button"
          className="button button--primary"
          onClick={onRetry}
        >
          Retry
        </button>
      </div>
    </main>
  );
}
