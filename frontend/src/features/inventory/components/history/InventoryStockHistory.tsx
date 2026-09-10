import "./InventoryStockHistory.css";
import type {
  InventoryStockHistoryActor,
  InventoryStockHistoryEntry,
} from "../../types";
import { getInventoryStockHistory } from "../../api";
import { useQuery } from "@tanstack/react-query";
import { queryKeys } from "../../../../api/queryKeys";
import Skeleton from "react-loading-skeleton";

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

  if (isPending) return <InventoryStockHistorySkeleton />;

  if (isError) {
    return (
      <p className="stock-history__message">Failed to load stock history</p>
    );
  }

  if (history.length === 0) {
    return <p className="stock-history__message">No stock history yet :(</p>;
  }

  return (
    <ul className="stock-history">
      {history.map((entry) => (
        <li key={entry.id} className="stock-history__entry">
          <strong className="stock-history__change">
            {getChangeLabel(entry)}
          </strong>

          <div className="stock-history__details">
            <strong className="stock-history__transition">
              {entry.stock_before} {"=>"} {entry.stock_after}
            </strong>

            <div className="stock-history__meta">
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
      return `-${entry.amount ?? 0}`;
    case "set":
      return `Set`;
    default:
      return entry.kind;
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
    <ul className="stock-history" aria-label="Loading stock history">
      {Array.from({ length: 3 }).map((_, index) => (
        <li
          key={index}
          className="stock-history__entry stock-history__entry--skeleton"
        >
          <div className="stock-history__change">
            <Skeleton width="2rem" />
          </div>

          <div className="stock-history__details">
            <div className="stock-history__transition">
              <Skeleton width="5rem" />
            </div>

            <div className="stock-history__meta">
              <Skeleton width="9rem" />
            </div>
          </div>
        </li>
      ))}
    </ul>
  );
}
