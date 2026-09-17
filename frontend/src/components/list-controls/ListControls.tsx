import { RotateCcw, Search, SlidersHorizontal, X } from "lucide-react";
import { useState, type ReactNode } from "react";

import styles from "./ListControls.module.css";

type Props = {
  search: string;
  onSearchChange: (value: string) => void;
  searchPlaceholder?: string;
  activeFilterCount?: number;
  onReset?: () => void;
  canReset?: boolean;
  children?: ReactNode;
};

export function ListControls({
  search,
  onSearchChange,
  searchPlaceholder = "Search...",
  activeFilterCount = 0,
  onReset,
  canReset = false,
  children,
}: Props) {
  const [showFilters, setShowFilters] = useState(false);

  return (
    <div className={styles.controls}>
      <div className={styles.main}>
        <div className={styles.search}>
          <Search className={styles.searchIcon} />

          <input
            type="search"
            value={search}
            placeholder={searchPlaceholder}
            onChange={(event) => onSearchChange(event.target.value)}
          />

          {search !== "" && (
            <button
              type="button"
              className={styles.searchClear}
              onClick={() => onSearchChange("")}
              aria-label="Clear search"
            >
              <X />
            </button>
          )}
        </div>

        {children && (
          <button
            type="button"
            className={`${styles.filterToggle} ${
              showFilters ? styles.filterToggleOpen : ""
            }`}
            onClick={() => setShowFilters((current) => !current)}
            aria-expanded={showFilters}
          >
            <SlidersHorizontal />

            <span>Filters</span>

            {activeFilterCount > 0 && (
              <span className={styles.filterCount}>{activeFilterCount}</span>
            )}
          </button>
        )}
      </div>

      {children && showFilters && (
        <div className={styles.filters}>
          {children}

          {onReset && (
            <button
              type="button"
              className={styles.reset}
              onClick={onReset}
              disabled={!canReset}
            >
              <RotateCcw />
              Reset
            </button>
          )}
        </div>
      )}
    </div>
  );
}
