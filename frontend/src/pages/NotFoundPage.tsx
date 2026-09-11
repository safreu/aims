import { Link } from "react-router-dom";

export function NotFoundPage() {
  return (
    <main className="not-found-page">
      <div className="not-found-page__content">
        <h1>Page not found</h1>
        <p>The page you were looking for does not exist.</p>

        <Link to="/households" className="button button--primary">
          Go to households
        </Link>
      </div>
    </main>
  );
}
