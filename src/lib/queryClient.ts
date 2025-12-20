import { QueryClient } from "@tanstack/react-query";

/**
 * Configured QueryClient for Tauri command caching.
 *
 * Configuration rationale:
 * - staleTime: 60s - Tauri commands return relatively stable data
 * - gcTime: 5 minutes - Keep unused queries in cache for reasonable time
 * - retry: 3 with exponential backoff - Handle transient IPC failures
 * - refetchOnWindowFocus: false - Desktop app, not browser; avoid unnecessary refetches
 */
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 60 * 1000,
      gcTime: 5 * 60 * 1000,
      retry: 3,
      refetchOnWindowFocus: false,
    },
  },
});
