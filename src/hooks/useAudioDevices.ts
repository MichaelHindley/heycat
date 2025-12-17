import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AudioInputDevice } from "../types/audio";

const DEFAULT_REFRESH_INTERVAL_MS = 5000;

export interface UseAudioDevicesOptions {
  /** Enable periodic refresh while hook is active (default: true) */
  autoRefresh?: boolean;
  /** Refresh interval in milliseconds (default: 5000) */
  refreshInterval?: number;
}

export interface UseAudioDevicesResult {
  devices: AudioInputDevice[];
  isLoading: boolean;
  error: Error | null;
  refresh: () => void;
}

/**
 * Hook for fetching available audio input devices from the backend.
 * Automatically loads devices on mount and provides a refresh function.
 * Re-fetches on window focus and periodically when autoRefresh is enabled.
 */
export function useAudioDevices(
  options: UseAudioDevicesOptions = {}
): UseAudioDevicesResult {
  const { autoRefresh = true, refreshInterval = DEFAULT_REFRESH_INTERVAL_MS } =
    options;

  const [devices, setDevices] = useState<AudioInputDevice[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const isFirstFetch = useRef(true);

  const fetchDevices = useCallback(async () => {
    // Only set loading on first fetch to avoid UI flickering on refresh
    if (isFirstFetch.current) {
      setIsLoading(true);
    }
    setError(null);
    try {
      const result = await invoke<AudioInputDevice[]>("list_audio_devices");
      setDevices((prev) => {
        // Log changes for debugging
        if (JSON.stringify(prev) !== JSON.stringify(result)) {
          console.log("[AudioDevices] Device list changed:", result);
        }
        return result;
      });
    } catch (e) {
      setError(e instanceof Error ? e : new Error(String(e)));
    } finally {
      setIsLoading(false);
      isFirstFetch.current = false;
    }
  }, []);

  // Initial fetch
  useEffect(() => {
    fetchDevices();
  }, [fetchDevices]);

  // Refresh on window focus
  useEffect(() => {
    const handleFocus = () => {
      console.log("[AudioDevices] Window focused, refreshing devices");
      fetchDevices();
    };

    window.addEventListener("focus", handleFocus);
    return () => window.removeEventListener("focus", handleFocus);
  }, [fetchDevices]);

  // Periodic refresh when autoRefresh enabled
  useEffect(() => {
    if (!autoRefresh) return;

    const interval = setInterval(() => {
      fetchDevices();
    }, refreshInterval);

    return () => clearInterval(interval);
  }, [autoRefresh, refreshInterval, fetchDevices]);

  return { devices, isLoading, error, refresh: fetchDevices };
}
