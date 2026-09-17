import { PRIORITIES, type Priority } from "../../../domain/priority";
import { PriorityIndicator } from "../../priority/PriorityIndicator";
import { Select, type SelectOption } from "../../select/Select";

export type PriorityFilterValue = "all" | Priority;

type Props = {
  value: PriorityFilterValue;
  onValueChange: (value: PriorityFilterValue) => void;
};

const options: SelectOption<PriorityFilterValue>[] = [
  {
    value: "all",
    label: "All priorities",
  },
  ...PRIORITIES.map((priority) => ({
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
      onValueChange={(value) => onValueChange(value as PriorityFilterValue)}
      ariaLabel="Filter by priority"
    />
  );
}
