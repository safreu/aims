import { useState, type SubmitEvent } from "react";
import { useToast } from "../../../../components/toast/ToastContext";
import type { InventoryItem, InventoryItemPriority } from "../../types";
import { updateInventoryItem } from "../../api";
import { InventoryItemFields } from "../fields/InventoryItemFields";

type ItemDraft = {
  name: string;
  categoryId: string | null;
  reorderThreshold: number | "";
  priority: InventoryItemPriority;
};

type Props = {
  householdId: string;
  item: InventoryItem;
  onChanged: () => Promise<void>;
};

export function InventoryItemDetails({ householdId, item, onChanged }: Props) {
  const { showToast } = useToast();

  const [draft, setDraft] = useState<ItemDraft | null>(null);
  const [isSaving, setIsSaving] = useState(false);

  const name = draft?.name ?? item?.name ?? "";
  const categoryId = draft?.categoryId ?? item?.category?.id ?? null;
  const reorderThreshold =
    draft?.reorderThreshold ?? item?.reorder_threshold ?? "";
  const priority = draft?.priority ?? item?.priority ?? "default";

  function updateDraft(changes: Partial<ItemDraft>) {
    setDraft((current) => ({
      name: current?.name ?? item.name,
      categoryId: current?.categoryId ?? item.category?.id ?? null,
      reorderThreshold:
        current?.reorderThreshold ?? item.reorder_threshold ?? "",
      priority: current?.priority ?? item.priority,
      ...changes,
    }));
  }

  function handleUpdate(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();

    setIsSaving(true);

    void updateInventoryItem(householdId, item.id, {
      name,
      category_id: categoryId,
      reorder_threshold: reorderThreshold === "" ? null : reorderThreshold,
      priority,
    })
      .then(async () => {
        await onChanged();
        setDraft(null);
        showToast("Inventory updated", "success");
      })
      .catch(() => showToast("Failed to update inventory", "error"))
      .finally(() => setIsSaving(false));
  }

  return (
    <form className="inventory-item-dialog__section" onSubmit={handleUpdate}>
      <h3>Details</h3>

      <InventoryItemFields
        householdId={householdId}
        name={name}
        categoryId={categoryId}
        reorderThreshold={reorderThreshold}
        priority={priority}
        onNameChange={(name) => updateDraft({ name })}
        onReorderThresholdChange={(reorderThreshold) =>
          updateDraft({ reorderThreshold })
        }
        onPriorityChange={(priority) => updateDraft({ priority })}
        onCategoryChange={(categoryId) => updateDraft({ categoryId })}
        disabled={isSaving}
      />

      <button
        type="submit"
        className="button button--primary"
        disabled={isSaving}
      >
        {isSaving ? "Saving..." : "Save changes"}
      </button>
    </form>
  );
}
