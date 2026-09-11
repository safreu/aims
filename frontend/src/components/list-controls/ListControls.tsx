import { useState, type ReactNode } from "react";
import "./ListControls.css";
import { RotateCcw, Search, SlidersHorizontal, X } from "lucide-react";

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
    <div className="list-controls">
      <div className="list-controls__main">
        <div className="list-controls__search">
          <Search className="list-controls__icon" />

          <input
            type="search"
            value={search}
            placeholder={searchPlaceholder}
            onChange={(event) => onSearchChange(event.target.value)}
          />

          {search !== "" && (
            <button
              type="button"
              className="list-controls__search-clear"
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
            className={`list-controls__filter-toggle ${showFilters ? "list-controls__filter-toggle--open" : ""}`}
            onClick={() => setShowFilters((current) => !current)}
          >
            <SlidersHorizontal />

            <span>Filters</span>

            {activeFilterCount > 0 && (
              <span className="list-controls__filter-count">
                {activeFilterCount}
              </span>
            )}
          </button>
        )}
      </div>

      {children && showFilters && (
        <div className="list-controls__filters">
          {children}

          {onReset && (
            <button
              type="button"
              className="list-controls__reset"
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
