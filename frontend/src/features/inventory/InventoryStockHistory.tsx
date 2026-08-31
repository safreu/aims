import { useEffect, useState } from "react";
import { getInventoryStockHistory } from "./api";
import type {
  InventoryStockHistoryActor,
  InventoryStockHistoryEntry,
} from "./types";

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
    setLoading(true);
    setError(null);

    void getInventoryStockHistory(householdId, itemId)
      .then((history) => setHistory(history))
      .catch(() => setError("Failed to load stock history"))
      .finally(() => setLoading(false));
  }, [householdId, itemId, version]);

  if (loading) {
    <p>Loading history...</p>;
  }

  if (error !== null) {
    return <p>{error}</p>;
  }

  if (history.length === 0) {
    return <p>No stock history yet :(</p>;
  }

  return (
    <ul>
      {history.map((entry) => (
        <li key={entry.id}>
          <strong>{getChangeLabel(entry)}</strong>
          {"   "}
          {entry.stock_before} {"=>"} {entry.stock_after}
          {" | "}
          {getActorName(entry.actor)}
          {" | "}
          {formatDate(entry.created_at)}
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
  return new Date(createdAt).toLocaleString();
}
