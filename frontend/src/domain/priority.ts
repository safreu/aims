import type { SelectOption } from "../components/select/Select";

export type Priority = "default" | "low" | "medium" | "high";

export const PRIORITIES: SelectOption<Priority>[] = [
  { value: "default", label: "Default" },
  { value: "low", label: "Low" },
  { value: "medium", label: "Medium" },
  { value: "high", label: "High" },
];
