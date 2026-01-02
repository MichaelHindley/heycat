import { useState, useCallback, useMemo } from "react";
import type { RecordingInfo } from "../pages/components/RecordingItem";

/**
 * Filter options for recordings list.
 */
export type FilterOption = "all" | "transcribed" | "pending";

/**
 * Sort options for recordings list.
 */
export type SortOption = "newest" | "oldest" | "longest" | "shortest";

/**
 * Configuration options for useRecordingsFilter hook.
 */
export interface UseRecordingsFilterOptions {
  /** The recordings to filter and sort */
  recordings: RecordingInfo[];
  /** Initial filter option */
  initialFilter?: FilterOption;
  /** Initial sort option */
  initialSort?: SortOption;
}

/**
 * Return type of the useRecordingsFilter hook.
 */
export interface UseRecordingsFilterReturn {
  /** Current search query */
  searchQuery: string;
  /** Set the search query */
  setSearchQuery: (query: string) => void;
  /** Current filter option */
  filterOption: FilterOption;
  /** Set the filter option */
  setFilterOption: (option: FilterOption) => void;
  /** Current sort option */
  sortOption: SortOption;
  /** Set the sort option */
  setSortOption: (option: SortOption) => void;
  /** Filtered and sorted recordings */
  filteredRecordings: RecordingInfo[];
  /** Whether there are any results */
  hasResults: boolean;
  /** Clear all filters and search */
  clearFilters: () => void;
  /** Whether any filters are active */
  hasActiveFilters: boolean;
}

/**
 * Hook for filtering and sorting recordings.
 *
 * Combines search, filter by transcription status, and sorting
 * into a unified hook for the recordings page.
 *
 * @example
 * const {
 *   searchQuery,
 *   setSearchQuery,
 *   filterOption,
 *   setFilterOption,
 *   sortOption,
 *   setSortOption,
 *   filteredRecordings,
 * } = useRecordingsFilter({ recordings });
 */
export function useRecordingsFilter(
  options: UseRecordingsFilterOptions
): UseRecordingsFilterReturn {
  const { recordings, initialFilter = "all", initialSort = "newest" } = options;

  const [searchQuery, setSearchQuery] = useState("");
  const [filterOption, setFilterOption] = useState<FilterOption>(initialFilter);
  const [sortOption, setSortOption] = useState<SortOption>(initialSort);

  const clearFilters = useCallback(() => {
    setSearchQuery("");
    setFilterOption("all");
    setSortOption("newest");
  }, []);

  const filteredRecordings = useMemo(() => {
    let result = [...recordings];

    // Apply search filter
    const trimmedQuery = searchQuery.trim().toLowerCase();
    if (trimmedQuery) {
      result = result.filter((recording) => {
        const searchableText = [
          recording.filename,
          recording.transcription,
          recording.active_window_app_name,
          recording.active_window_title,
        ]
          .filter(Boolean)
          .join(" ")
          .toLowerCase();
        return searchableText.includes(trimmedQuery);
      });
    }

    // Apply transcription filter
    if (filterOption === "transcribed") {
      result = result.filter((r) => Boolean(r.transcription));
    } else if (filterOption === "pending") {
      result = result.filter((r) => !r.transcription && !r.error);
    }

    // Apply sorting
    result.sort((a, b) => {
      switch (sortOption) {
        case "newest":
          return (
            new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
          );
        case "oldest":
          return (
            new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
          );
        case "longest":
          return b.duration_secs - a.duration_secs;
        case "shortest":
          return a.duration_secs - b.duration_secs;
        default:
          return 0;
      }
    });

    return result;
  }, [recordings, searchQuery, filterOption, sortOption]);

  const hasResults = filteredRecordings.length > 0;
  const hasActiveFilters =
    searchQuery.trim() !== "" || filterOption !== "all" || sortOption !== "newest";

  return {
    searchQuery,
    setSearchQuery,
    filterOption,
    setFilterOption,
    sortOption,
    setSortOption,
    filteredRecordings,
    hasResults,
    clearFilters,
    hasActiveFilters,
  };
}
