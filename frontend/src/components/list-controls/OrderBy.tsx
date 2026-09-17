import { Select, type SelectOption } from "../select/Select";

export type OrderByValue =
  | "default"
  | "quantity-desc"
  | "quantity-asc"
  | "priority-desc"
  | "priority-asc";

type Props = {
  value: OrderByValue;
  onValueChange: (value: OrderByValue) => void;
};

const options: SelectOption<OrderByValue>[] = [
  { value: "default", label: "Default" },
  { value: "quantity-desc", label: "Amount: High to low" },
  { value: "quantity-asc", label: "Amount: Low to high" },
  { value: "priority-desc", label: "Priority: High to low" },
  { value: "priority-asc", label: "Priority: Low to high" },
];

export function OrderBy({ value, onValueChange }: Props) {
  return (
    <Select
      value={value}
      options={options}
      onValueChange={onValueChange}
      ariaLabel="Order shopping list by"
    />
  );
}
