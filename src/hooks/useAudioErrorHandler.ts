import { useState, useEffect, useCallback } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { AudioDeviceError } from "../types/audio";

/** Return type of the useAudioErrorHandler hook */
export interface UseAudioErrorHandlerReturn {
  /** Current error, or null if no error */
  error: AudioDeviceError | null;
  /** Clear the current error */
  clearError: () => void;
}

/**
 * Custom hook for handling audio device errors
 *
 * Listens for 'audio_device_error' events from the backend and maintains
 * the current error state. The error state can be cleared by calling clearError.
 */
export function useAudioErrorHandler(): UseAudioErrorHandlerReturn {
  const [error, setError] = useState<AudioDeviceError | null>(null);

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  useEffect(() => {
    const unlistenFns: UnlistenFn[] = [];

    /* v8 ignore start -- @preserve */
    const setupListener = async () => {
      const unlisten = await listen<AudioDeviceError>(
        "audio_device_error",
        (event) => {
          console.error("[AudioError]", event.payload);
          setError(event.payload);
        }
      );
      unlistenFns.push(unlisten);
    };

    setupListener();
    /* v8 ignore stop */

    return () => {
      /* v8 ignore start -- @preserve */
      unlistenFns.forEach((unlisten) => unlisten());
      /* v8 ignore stop */
    };
  }, []);

  return { error, clearError };
}
