import { Check, ChevronDown, Plus } from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuItem,
  DropDownMenuSeparator,
} from "../../../../components/dropdown-menu/DropdownMenu";
import type { InventoryItemCategory } from "../../types";
import { createPortal } from "react-dom";

import "./CategorySelect.css";
import { useEffect, useState } from "react";
import { getInventoryCategories } from "../../api";
import { CreateInventoryCategoryDialog } from "../dialogs/CreateInventoryCategoryDialog";

type CategorySelectProps = {
  householdId: string;
  value: string | null;
  onValueChange: (categoryId: string | null) => void;
};

export function CategorySelect({
  householdId,
  value,
  onValueChange,
}: CategorySelectProps) {
  const [categories, setCategories] = useState<InventoryItemCategory[]>([]);
  const [showCreateCategoryDialog, setShowCreateCategoryDialog] =
    useState(false);

  const [search, setSearch] = useState("");
  const filteredCategories = categories.filter((category) =>
    category.name.toLowerCase().includes(search.trim().toLowerCase()),
  );

  async function refreshCategories() {
    const categories = await getInventoryCategories(householdId);
    setCategories(categories);
  }

  useEffect(() => {
    async function loadCategories() {
      const categories = await getInventoryCategories(householdId);
      setCategories(categories);
    }

    void loadCategories();
  }, [householdId]);

  async function handleCategoryCreated(categoryId: string) {
    await refreshCategories();

    onValueChange(categoryId);
    setShowCreateCategoryDialog(false);
  }

  const selectedCategory = categories.find((category) => category.id === value);

  return (
    <>
      <DropdownMenu
        portal={false}
        onOpenChange={(open) => {
          if (!open) setSearch("");
        }}
        trigger={
          <button
            type="button"
            className="category-select"
            aria-label="Select category"
          >
            <span className="category-select__label">
              {selectedCategory?.name ?? "No category"}
            </span>

            <ChevronDown className="category-select__chevron" />
          </button>
        }
      >
        <div className="category-select__search">
          <input
            type="search"
            placeholder="Search categories..."
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            onKeyDown={(event) => event.stopPropagation()}
          />
        </div>

        <div className="category-select__options">
          <DropdownMenuItem onSelect={() => onValueChange(null)}>
            <span className="category-select__name">No category</span>

            {value === null && <Check className="category-select__selected" />}
          </DropdownMenuItem>

          {filteredCategories.map((category) => (
            <DropdownMenuItem
              key={category.id}
              onSelect={() => onValueChange(category.id)}
            >
              <span className="category-select__name">{category.name}</span>

              {category.id === value && (
                <Check className="category-select__selected" />
              )}
            </DropdownMenuItem>
          ))}

          {filteredCategories.length === 0 && (
            <div className="category-select__empty">No categories found</div>
          )}
        </div>

        <DropDownMenuSeparator />

        <DropdownMenuItem
          className="category-select__create"
          onSelect={() => setShowCreateCategoryDialog(true)}
        >
          <Plus />
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
