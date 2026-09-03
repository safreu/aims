import { useEffect, useState } from "react";
import "./InventoryStockHistory.css";
import type {
  InventoryStockHistoryActor,
  InventoryStockHistoryEntry,
} from "../../types";
import { getInventoryStockHistory } from "../../api";

type InventoryStockHistoryProps = {
  householdId: string;
  itemId: string;
  version: number;
};

export function InventoryStockHistory({
  householdId,
  itemId,
  version,
}: InventoryStockHistoryProps) {
  const [history, setHistory] = useState<InventoryStockHistoryEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function loadHistory() {
      setLoading(true);
      setError(null);

      await getInventoryStockHistory(householdId, itemId)
        .then((history) => setHistory(history))
        .catch(() => setError("Failed to load stock history"))
        .finally(() => setLoading(false));
    }

    void loadHistory();
  }, [householdId, itemId, version]);

  if (loading) {
    return <p className="stock-history__message">Loading history...</p>;
  }

  if (error !== null) {
    return <p className="stock-history__message">{error}</p>;
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
