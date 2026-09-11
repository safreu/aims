import {
  BadgeAlert,
  Circle,
  CircleAlert,
  Minus,
  type LucideIcon,
} from "lucide-react";
import type { InventoryItemPriority } from "../../types";
import "./PriorityIndicator.css";

type Props = {
  priority: InventoryItemPriority;
  showLabel?: boolean;
};

type PriorityConfig = {
  label: string;
  icon: LucideIcon;
};

const PRIORITY_CONFIG: Record<InventoryItemPriority, PriorityConfig> = {
  default: {
    label: "Default",
    icon: Minus,
  },
  low: {
    label: "Low",
    icon: Circle,
  },
  medium: {
    label: "Medium",
    icon: CircleAlert,
  },
  high: {
    label: "High",
    icon: BadgeAlert,
  },
};

export function PriorityIndicator({ priority, showLabel = false }: Props) {
  const { label, icon: Icon } = PRIORITY_CONFIG[priority];
  return (
    <span
      className={`priority-indicator priority-indicator--${priority}`}
      title={`${label} priority`}
      aria-label={`${label} priority`}
    >
      <Icon className="priority-indicator__icon" />

      {showLabel && <span className="priority-indicator__label">{label}</span>}
    </span>
  );
}
