import {
  BadgeAlert,
  Circle,
  CircleAlert,
  Minus,
  type LucideIcon,
} from "lucide-react";

import type { Priority } from "../../domain/priority";

import styles from "./PriorityIndicator.module.css";

type Props = {
  priority: Priority;
  showLabel?: boolean;
};

type PriorityConfig = {
  label: string;
  icon: LucideIcon;
};

const PRIORITY_CONFIG: Record<Priority, PriorityConfig> = {
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
      className={`${styles.indicator} ${styles[priority]}`}
      title={`${label} priority`}
      aria-label={`${label} priority`}
    >
      <Icon className={styles.icon} aria-hidden="true" />

      {showLabel && <span className={styles.label}>{label}</span>}
    </span>
  );
}
