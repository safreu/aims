import { useCategories } from "../../../features/inventory/components/categories/CategoryContex";
import { Select, type SelectOption } from "../../select/Select";

export type CategoryFilterValue = string;

type Props = {
  value: CategoryFilterValue;
  onValueChange: (value: CategoryFilterValue) => void;
};

export function CategoryFilter({ value, onValueChange }: Props) {
  const { categories } = useCategories();

  const options: SelectOption<string>[] = [
    { value: "all", label: "All categories" },
    { value: "uncategorized", label: "Uncategorized" },
    ...categories.map((category) => ({
      value: category.id,
      label: category.name,
    })),
  ];

  return (
    <Select
      value={value}
      options={options}
      onValueChange={onValueChange}
      ariaLabel="Filter by category"
    />
  );
}
