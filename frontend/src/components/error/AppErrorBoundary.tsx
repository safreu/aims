import { Component, type ReactNode } from "react";
import type { ErrorInfo } from "react-dom/client";

import { appLogger } from "../../diagnostics/logger";
import styles from "./AppErrorBoundary.module.css";

type Props = {
  children: ReactNode;
};

type State = {
  hasError: boolean;
};

export class AppErrorBoundary extends Component<Props, State> {
  state: State = {
    hasError: false,
  };

  static getDerivedStateFromError(): State {
    return {
      hasError: true,
    };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    appLogger.error("Unhandled React error", {
      error: error.message,
      stack: error.stack,
      componentStack: info.componentStack,
    });
  }

  handleReload = () => {
    window.location.reload();
  };

  render() {
    if (this.state.hasError) {
      return (
        <main className={styles.container}>
          <div className={styles.content}>
            <h1 className={styles.title}>Something went wrong</h1>

            <p className={styles.description}>
              Aims encountered an unexpected error. Reload the app to try again.
            </p>

            <button
              type="button"
              className="button button--primary"
              onClick={this.handleReload}
            >
              Reload
            </button>
          </div>
        </main>
      );
    }

    return this.props.children;
  }
}
