import "./EmptyState.css";

export interface EmptyStateProps {
  hasFiltersActive: boolean;
  onClearFilters?: () => void;
  className?: string;
}

export function EmptyState({
  hasFiltersActive,
  onClearFilters,
  className = "",
}: EmptyStateProps) {
  const title = hasFiltersActive
    ? "No recordings match your filters"
    : "No recordings yet";

  const description = hasFiltersActive
    ? "Try adjusting your filters to see more results"
    : "Make your first recording to see it here";

  return (
    <div
      className={`empty-state ${className}`.trim()}
      role="status"
      aria-live="polite"
    >
      <span className="empty-state__icon" aria-hidden="true">
        {hasFiltersActive ? "🔍" : "🎙️"}
      </span>
      <h3 className="empty-state__title">{title}</h3>
      <p className="empty-state__description">{description}</p>
      {hasFiltersActive && onClearFilters && (
        <button
          className="empty-state__clear-button"
          onClick={onClearFilters}
          type="button"
        >
          Clear filters
        </button>
      )}
    </div>
  );
}
