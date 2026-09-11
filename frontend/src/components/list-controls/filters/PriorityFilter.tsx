import { PriorityIndicator } from "../../../features/inventory/components/priority/PriorityIndicator";
import { INVENTORY_PRIORITIES } from "../../../features/inventory/priorities";
import { Select, type SelectOption } from "../../select/Select";

export type PriorityFilterValue = string;

type Props = {
  value: PriorityFilterValue;
  onValueChange: (value: PriorityFilterValue) => void;
};

const options: SelectOption<string>[] = [
  { value: "all", label: "All Priorities" },
  ...INVENTORY_PRIORITIES.map((priority) => ({
    value: priority.value,
    label: priority.label,
    content: <PriorityIndicator priority={priority.value} showLabel />,
  })),
];
export function PriorityFilter({ value, onValueChange }: Props) {
  return (
    <Select
      value={value}
      options={options}
      onValueChange={onValueChange}
      ariaLabel="Filter by Priority"
    />
  );
}
