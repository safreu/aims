import { useQuery } from "@tanstack/react-query";
import Skeleton from "react-loading-skeleton";

import { queryKeys } from "../../../../api/queryKeys";
import { getInventoryStockHistory } from "../../api";
import type {
  InventoryStockHistoryActor,
  InventoryStockHistoryEntry,
} from "../../types";

import styles from "./InventoryStockHistory.module.css";

type InventoryStockHistoryProps = {
  householdId: string;
  itemId: string;
};

export function InventoryStockHistory({
  householdId,
  itemId,
}: InventoryStockHistoryProps) {
  const {
    data: history = [],
    isPending,
    isError,
  } = useQuery({
    queryKey: queryKeys.inventory.history(householdId, itemId),
    queryFn: () => getInventoryStockHistory(householdId, itemId),
  });

  if (isPending) {
    return <InventoryStockHistorySkeleton />;
  }

  if (isError) {
    return (
      <p className={styles.message} role="alert">
        Failed to load stock history
      </p>
    );
  }

  if (history.length === 0) {
    return <p className={styles.message}>No stock history yet</p>;
  }

  return (
    <ul className={styles.history}>
      {history.map((entry) => (
        <li key={entry.id} className={styles.entry}>
          <strong className={styles.change}>{getChangeLabel(entry)}</strong>

          <div className={styles.details}>
            <strong className={styles.transition}>
              {entry.stock_before} → {entry.stock_after}
            </strong>

            <div className={styles.meta}>
              <span>{getActorName(entry.actor)}</span>

              <span aria-hidden="true">·</span>

              <time dateTime={entry.created_at}>
                {formatDate(entry.created_at)}
              </time>
            </div>
          </div>
        </li>
      ))}
    </ul>
  );
}

function getActorName(actor: InventoryStockHistoryActor): string {
  switch (actor.type) {
    case "user":
      return actor.display_name;

    case "device":
      return actor.name;

    case "system":
      return "System";
  }
}

function getChangeLabel(entry: InventoryStockHistoryEntry): string {
  switch (entry.kind) {
    case "increase":
      return `+${entry.amount ?? 0}`;

    case "decrease":
      return `−${entry.amount ?? 0}`;

    case "set":
      return "Set";
  }
}

function formatDate(createdAt: string): string {
  return new Date(createdAt).toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  });
}

function InventoryStockHistorySkeleton() {
  return (
    <ul className={styles.history} aria-label="Loading stock history">
      {Array.from({ length: 3 }).map((_, index) => (
        <li key={index} className={styles.entry}>
          <div className={styles.change}>
            <Skeleton width="2rem" />
          </div>

          <div className={styles.details}>
            <div className={styles.transition}>
              <Skeleton width="5rem" />
            </div>

            <div className={styles.meta}>
              <Skeleton width="9rem" />
            </div>
          </div>
        </li>
      ))}
    </ul>
  );
}
