import { useEffect, useState, type SubmitEvent } from "react";
import { useParams } from "react-router-dom";
import type {
  InventoryItem,
  InventoryItemCategory,
  InventoryItemPriority,
} from "../features/inventory/types";
import {
  createInventoryCategory,
  createInventoryItem,
  getInventoryCategories,
  getInventoryItems,
} from "../features/inventory/api";
import { InventoryItemRow } from "../features/inventory/InventoryItemRow";
import { ArchivedInventoryItemRow } from "../features/inventory/ArchivedInventoryItemRow";

export function InventoryPage() {
  const { householdId } = useParams();

  if (householdId === undefined) {
    throw new Error("InventoryPage requires a householdId");
  }

  const resolvedHouseholdId = householdId;

  const [items, setItems] = useState<InventoryItem[]>([]);
  const [archivedItems, setArchivedItems] = useState<InventoryItem[]>([]);
  const [categories, setCategories] = useState<InventoryItemCategory[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [name, setName] = useState("");
  const [categoryId, setCategoryId] = useState<string>("");
  const [categoryName, setCategoryName] = useState("");
  const [currentStock, setCurrentStock] = useState(0);
  const [reorderThreshold, setReorderThreshold] = useState(0);
  const [priority, setPriority] = useState<InventoryItemPriority>("default");

  const [createError, setCreateError] = useState<string | null>(null);
  const [createCategoryError, setCreateCategoryError] = useState<string | null>(
    null,
  );

  async function refreshInventory() {
    const [items, archivedItems] = await Promise.all([
      getInventoryItems(resolvedHouseholdId),
      getInventoryItems(resolvedHouseholdId, "archived"),
    ]);

    setItems(items);
    setArchivedItems(archivedItems);
  }

  function handleCreateInventoryItem(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();
    setCreateError(null);

    void createInventoryItem(resolvedHouseholdId, {
      name,
      category_id: categoryId === "" ? null : categoryId,
      current_stock: currentStock,
      reorder_threshold: reorderThreshold,
      priority,
    })
      .then(async () => {
        setName("");
        setCurrentStock(0);
        setReorderThreshold(0);

        await refreshInventory();
      })
      .catch(() => setCreateError("Failed to create inventory item"));
  }

  function handleCreateInventoryCategory(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();
    setCreateCategoryError(null);

    void createInventoryCategory(resolvedHouseholdId, {
      name: categoryName,
    })
      .then(async () => {
        setCategoryName("");

        const categories = await getInventoryCategories(resolvedHouseholdId);
        setCategories(categories);
      })
      .catch(() => setCreateError("Failed to create category"));
  }

  useEffect(() => {
    void Promise.all([
      getInventoryItems(resolvedHouseholdId),
      getInventoryItems(resolvedHouseholdId, "archived"),
      getInventoryCategories(resolvedHouseholdId),
    ])
      .then(([items, archivedItems, categories]) => {
        setItems(items);
        setArchivedItems(archivedItems);
        setCategories(categories);
      })
      .catch(() => setError("Failed to load inventory"))
      .finally(() => setLoading(false));
  }, [resolvedHouseholdId]);

  if (loading) {
    return <p>Loading Inventory...</p>;
  }

  if (error !== null) {
    return <p>{error}</p>;
  }

  return (
    <main>
      <h1>Inventory</h1>

      <form onSubmit={handleCreateInventoryCategory}>
        <label htmlFor="category-name">name</label>
        <input
          id="category-name"
          type="text"
          value={categoryName}
          onChange={(event) => setCategoryName(event.target.value)}
          required
        />

        <button type="submit">Create category</button>

        {createCategoryError !== null && <p>{createCategoryError}</p>}
      </form>

      <br />

      <form onSubmit={handleCreateInventoryItem}>
        <div>
          <label htmlFor="inventory-name">Name</label>
          <input
            id="inventory-name"
            type="text"
            value={name}
            onChange={(event) => setName(event.target.value)}
            required
          />
        </div>

        <div>
          <label htmlFor="category">Category</label>
          <select
            id="category"
            value={categoryId}
            onChange={(event) => setCategoryId(event.target.value)}
          >
            <option value="">No category</option>
            {categories.map((category) => (
              <option key={category.id} value={category.id}>
                {category.name}
              </option>
            ))}
          </select>
        </div>

        <div>
          <label htmlFor="current-stock">Current stock</label>
          <input
            id="current-stock"
            type="number"
            min="0"
            value={currentStock}
            onChange={(event) => setCurrentStock(Number(event.target.value))}
            required
          />
        </div>

        <div>
          <label htmlFor="reorder-threshold">Reorder threshold</label>
          <input
            id="reorder-threshold"
            type="number"
            min="0"
            value={reorderThreshold}
            onChange={(event) =>
              setReorderThreshold(Number(event.target.value))
            }
            required
          />
        </div>

        <div>
          <label htmlFor="priority">Set Priority</label>
          <select
            id="priority"
            value={priority}
            onChange={(event) =>
              setPriority(event.target.value as InventoryItemPriority)
            }
          >
            <option value="default">Default</option>
            <option value="low">Low</option>
            <option value="medium">Medium</option>
            <option value="high">High</option>
          </select>
        </div>

        <button type="submit">Create Item</button>

        {createError !== null && <p>{createError}</p>}
      </form>

      <br />

      {items.length === 0 ? (
        <p>No inventory items :(</p>
      ) : (
        <ul>
          {items.map((item) => (
            <InventoryItemRow
              key={item.id}
              householdId={resolvedHouseholdId}
              item={item}
              categories={categories}
              onChanged={refreshInventory}
            />
          ))}
        </ul>
      )}

      {archivedItems.length === 0 ? (
        <p>No archived items :(</p>
      ) : (
        <ul>
          {archivedItems.map((item) => (
            <ArchivedInventoryItemRow
              key={item.id}
              householdId={resolvedHouseholdId}
              item={item}
              onChanged={refreshInventory}
            />
          ))}
        </ul>
      )}
    </main>
  );
}
