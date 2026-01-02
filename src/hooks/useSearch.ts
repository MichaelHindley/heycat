import { useState, useCallback, useMemo } from "react";

/**
 * Configuration options for useSearch hook.
 */
export interface UseSearchOptions<T> {
  /** The items to search through */
  items: T[];
  /** Function to get searchable text from an item */
  getSearchableText: (item: T) => string;
  /** Optional initial search query */
  initialQuery?: string;
}

/**
 * Return type of the useSearch hook.
 */
export interface UseSearchReturn<T> {
  /** Current search query */
  query: string;
  /** Set the search query */
  setQuery: (query: string) => void;
  /** Items filtered by the current query */
  filteredItems: T[];
  /** Whether there are any results */
  hasResults: boolean;
  /** Clear the search query */
  clearSearch: () => void;
  /** Whether a search is active (query is not empty) */
  isSearching: boolean;
}

/**
 * Hook for filtering a list of items by a search query.
 *
 * Provides case-insensitive search with debounce-friendly API.
 *
 * @example
 * const search = useSearch({
 *   items: entries,
 *   getSearchableText: (entry) => `${entry.trigger} ${entry.expansion}`,
 * });
 *
 * <Input
 *   value={search.query}
 *   onChange={(e) => search.setQuery(e.target.value)}
 *   placeholder="Search..."
 * />
 *
 * {search.filteredItems.map((item) => ...)}
 */
export function useSearch<T>(options: UseSearchOptions<T>): UseSearchReturn<T> {
  const { items, getSearchableText, initialQuery = "" } = options;

  const [query, setQueryState] = useState(initialQuery);

  const setQuery = useCallback((newQuery: string) => {
    setQueryState(newQuery);
  }, []);

  const clearSearch = useCallback(() => {
    setQueryState("");
  }, []);

  const filteredItems = useMemo(() => {
    const trimmedQuery = query.trim().toLowerCase();
    if (!trimmedQuery) {
      return items;
    }

    return items.filter((item) => {
      const searchableText = getSearchableText(item).toLowerCase();
      return searchableText.includes(trimmedQuery);
    });
  }, [items, query, getSearchableText]);

  const hasResults = filteredItems.length > 0;
  const isSearching = query.trim().length > 0;

  return {
    query,
    setQuery,
    filteredItems,
    hasResults,
    clearSearch,
    isSearching,
  };
}
