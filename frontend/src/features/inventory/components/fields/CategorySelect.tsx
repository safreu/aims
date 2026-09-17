import { Check, ChevronDown, Plus } from "lucide-react";
import { useState } from "react";
import { createPortal } from "react-dom";

import {
  DropdownMenu,
  DropdownMenuItem,
  DropdownMenuSeparator,
} from "../../../../components/dropdown-menu/DropdownMenu";
import { useCategories } from "../categories/CategoryContext";
import { CreateInventoryCategoryDialog } from "../dialogs/CreateInventoryCategoryDialog";

import styles from "./CategorySelect.module.css";

type CategorySelectProps = {
  householdId: string;
  value: string | null;
  onValueChange: (categoryId: string | null) => void;
  disabled?: boolean;
};

export function CategorySelect({
  householdId,
  value,
  onValueChange,
  disabled = false,
}: CategorySelectProps) {
  const { categories, refreshCategories } = useCategories();

  const [showCreateCategoryDialog, setShowCreateCategoryDialog] =
    useState(false);
  const [search, setSearch] = useState("");

  const normalizedSearch = search.trim().toLowerCase();

  const filteredCategories = categories.filter((category) =>
    category.name.toLowerCase().includes(normalizedSearch),
  );

  const selectedCategory = categories.find((category) => category.id === value);

  async function handleCategoryCreated(categoryId: string) {
    await refreshCategories();

    onValueChange(categoryId);
    setShowCreateCategoryDialog(false);
  }

  return (
    <>
      <DropdownMenu
        portal={false}
        onOpenChange={(open) => {
          if (!open) {
            setSearch("");
          }
        }}
        trigger={
          <button
            type="button"
            className={styles.trigger}
            disabled={disabled}
            aria-label="Select category"
          >
            <span className={styles.label}>
              {selectedCategory?.name ?? "No category"}
            </span>

            <ChevronDown className={styles.chevron} aria-hidden="true" />
          </button>
        }
      >
        <div className={styles.search}>
          <input
            type="search"
            placeholder="Search categories..."
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            onKeyDown={(event) => event.stopPropagation()}
            aria-label="Search categories"
          />
        </div>

        <div className={styles.options}>
          <DropdownMenuItem onSelect={() => onValueChange(null)}>
            <span className={styles.name}>No category</span>

            {value === null && (
              <Check className={styles.selected} aria-hidden="true" />
            )}
          </DropdownMenuItem>

          {filteredCategories.map((category) => (
            <DropdownMenuItem
              key={category.id}
              onSelect={() => onValueChange(category.id)}
            >
              <span className={styles.name}>{category.name}</span>

              {category.id === value && (
                <Check className={styles.selected} aria-hidden="true" />
              )}
            </DropdownMenuItem>
          ))}

          {filteredCategories.length === 0 && (
            <div className={styles.empty}>No categories found</div>
          )}
        </div>

        <DropdownMenuSeparator />

        <DropdownMenuItem
          className={styles.create}
          onSelect={() => setShowCreateCategoryDialog(true)}
        >
          <Plus aria-hidden="true" />
          <span>Create category</span>
        </DropdownMenuItem>
      </DropdownMenu>

      {showCreateCategoryDialog &&
        createPortal(
          <CreateInventoryCategoryDialog
            householdId={householdId}
            onCreated={handleCategoryCreated}
            onClose={() => setShowCreateCategoryDialog(false)}
          />,
          document.body,
        )}
    </>
  );
}
