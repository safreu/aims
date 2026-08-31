import { restoreInventoryItem } from "./api";
import type { InventoryItem } from "./types";

type ArchivedInventoryItemProps = {
  householdId: string;
  item: InventoryItem;
  onChanged: () => Promise<void>;
};

export function ArchivedInventoryItemRow({
  householdId,
  item,
  onChanged,
}: ArchivedInventoryItemProps) {
  function handleRestore() {
    void restoreInventoryItem(householdId, item.id).then(() => onChanged());
  }

  return (
    <li>
      {item.name}

      <button type="button" onClick={handleRestore}>
        Restore
      </button>
    </li>
  );
}
