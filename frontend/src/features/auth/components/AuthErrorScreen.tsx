type Props = {
  onRetry: () => void;
};

export function AuthErrorScreen({ onRetry }: Props) {
  return (
    <main className="auth-status">
      <div className="auth-status__content">
        <h1>Unable to connect</h1>

        <p>
          Aims could not determine your login status. Check your connection and
          try again
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
