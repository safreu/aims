import { useState, type SubmitEvent } from "react";

import { useToast } from "../../../../components/toast/ToastContext";
import { updateInventoryItem } from "../../api";
import type { InventoryItem } from "../../types";
import { InventoryItemFields } from "../fields/InventoryItemFields";
import type { Priority } from "../../../../domain/priority";

type ItemDraft = {
  name: string;
  categoryId: string | null;
  reorderThreshold: number | "";
  priority: Priority;
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

  const name = draft?.name ?? item.name;
  const categoryId = draft?.categoryId ?? item.category?.id ?? null;
  const reorderThreshold =
    draft?.reorderThreshold ?? item.reorder_threshold ?? "";
  const priority = draft?.priority ?? item.priority;

  const trimmedName = name.trim();

  const nameError = trimmedName === "" ? "Name is required" : undefined;

  const reorderThresholdError =
    reorderThreshold !== "" &&
    (!Number.isInteger(reorderThreshold) || reorderThreshold < 0)
      ? "Reorder threshold must be a non-negative whole number"
      : undefined;

  const hasChanges =
    trimmedName !== item.name ||
    categoryId !== (item.category?.id ?? null) ||
    reorderThreshold !== (item.reorder_threshold ?? "") ||
    priority !== item.priority;

  const isValid =
    nameError === undefined && reorderThresholdError === undefined;

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

    if (!hasChanges || !isValid) return;

    setIsSaving(true);

    void updateInventoryItem(householdId, item.id, {
      name: trimmedName,
      category_id: categoryId,
      reorder_threshold: reorderThreshold === "" ? null : reorderThreshold,
      priority,
    })
      .then(async () => {
        await onChanged();
        setDraft(null);
        showToast("Inventory item updated", "success");
      })
      .catch(() => showToast("Failed to update inventory item", "error"))
      .finally(() => setIsSaving(false));
  }

  return (
    <form className="dialog__section" onSubmit={handleUpdate}>
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
        nameError={draft !== null ? nameError : undefined}
        reorderThresholdError={
          draft !== null ? reorderThresholdError : undefined
        }
        disabled={isSaving}
      />

      <button
        type="submit"
        className="button button--primary"
        disabled={isSaving || !hasChanges || !isValid}
      >
        {isSaving ? "Saving..." : "Save changes"}
      </button>
    </form>
  );
}
