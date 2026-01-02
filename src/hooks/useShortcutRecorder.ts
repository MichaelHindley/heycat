import { useState, useCallback, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import {
  formatBackendKeyForDisplay,
  formatBackendKeyForBackend,
  isValidHotkey,
} from "../lib/formatting";
import type { CapturedKeyEvent } from "../lib/constants";

/**
 * Recorded shortcut data.
 */
export interface RecordedShortcut {
  /** Display format (e.g., "⌘⇧R") */
  display: string;
  /** Backend format (e.g., "Command+Shift+R") */
  backend: string;
}

/**
 * Configuration options for useShortcutRecorder hook.
 */
export interface UseShortcutRecorderOptions {
  /** Whether to distinguish left/right modifiers */
  distinguishLeftRight?: boolean;
  /** Callback when a valid shortcut is recorded */
  onRecorded?: (shortcut: RecordedShortcut) => void;
}

/**
 * Return type of the useShortcutRecorder hook.
 */
export interface UseShortcutRecorderReturn {
  /** Whether currently recording */
  isRecording: boolean;
  /** The recorded shortcut, if any */
  recordedShortcut: RecordedShortcut | null;
  /** Permission error message, if any */
  permissionError: string | null;
  /** Start recording a shortcut */
  startRecording: () => Promise<void>;
  /** Stop recording without saving */
  stopRecording: () => Promise<void>;
  /** Clear the recorded shortcut */
  clearRecordedShortcut: () => void;
  /** Open accessibility preferences (for permission errors) */
  openAccessibilityPreferences: () => Promise<void>;
}

/**
 * Hook for recording keyboard shortcuts using native capture.
 *
 * Uses the backend CGEventTap to capture all keyboard events including
 * fn key and media keys. Handles permission errors gracefully.
 *
 * @example
 * const recorder = useShortcutRecorder({
 *   distinguishLeftRight: settings.shortcuts.distinguishLeftRight,
 *   onRecorded: (shortcut) => {
 *     onSave(shortcut.display, shortcut.backend);
 *   },
 * });
 *
 * <Button onClick={recorder.startRecording}>
 *   {recorder.isRecording ? "Recording..." : "Record Shortcut"}
 * </Button>
 */
export function useShortcutRecorder(
  options: UseShortcutRecorderOptions = {}
): UseShortcutRecorderReturn {
  const { distinguishLeftRight = false, onRecorded } = options;

  const [isRecording, setIsRecording] = useState(false);
  const [recordedShortcut, setRecordedShortcut] = useState<RecordedShortcut | null>(
    null
  );
  const [permissionError, setPermissionError] = useState<string | null>(null);

  const unlistenRef = useRef<UnlistenFn | null>(null);

  // Start backend keyboard capture
  const startCapture = useCallback(async () => {
    try {
      await invoke("start_shortcut_recording");
      setPermissionError(null);
    } catch (error) {
      const errorMessage = String(error);
      if (errorMessage.includes("Accessibility permission")) {
        setPermissionError(errorMessage);
      }
      setIsRecording(false);
      throw error;
    }
  }, []);

  // Stop backend keyboard capture
  const stopCapture = useCallback(async () => {
    try {
      await invoke("stop_shortcut_recording");
    } catch (error) {
      console.error("[heycat] Failed to stop keyboard capture:", error);
    }
  }, []);

  const startRecording = useCallback(async () => {
    setIsRecording(true);
    setRecordedShortcut(null);
    setPermissionError(null);
    try {
      await startCapture();
    } catch {
      setIsRecording(false);
    }
  }, [startCapture]);

  const stopRecording = useCallback(async () => {
    setIsRecording(false);
    await stopCapture();
    if (unlistenRef.current) {
      unlistenRef.current();
      unlistenRef.current = null;
    }
  }, [stopCapture]);

  const clearRecordedShortcut = useCallback(() => {
    setRecordedShortcut(null);
  }, []);

  const openAccessibilityPreferences = useCallback(async () => {
    try {
      await invoke("open_accessibility_preferences");
    } catch (error) {
      console.error("[heycat] Failed to open preferences:", error);
    }
  }, []);

  // Handle backend key events when recording
  useEffect(() => {
    if (!isRecording) {
      // Clean up listener when not recording
      if (unlistenRef.current) {
        unlistenRef.current();
        unlistenRef.current = null;
      }
      return;
    }

    let isMounted = true;

    const setupListener = async () => {
      const unlisten = await listen<CapturedKeyEvent>(
        "shortcut_key_captured",
        (event) => {
          if (!isMounted) return;

          const keyEvent = event.payload;

          if (isValidHotkey(keyEvent)) {
            const display = formatBackendKeyForDisplay(
              keyEvent,
              distinguishLeftRight
            );
            const backend = formatBackendKeyForBackend(keyEvent);
            const shortcut = { display, backend };

            setRecordedShortcut(shortcut);
            setIsRecording(false);
            stopCapture();

            if (onRecorded) {
              onRecorded(shortcut);
            }
          }
        }
      );

      unlistenRef.current = unlisten;
    };

    setupListener();

    return () => {
      isMounted = false;
      if (unlistenRef.current) {
        unlistenRef.current();
        unlistenRef.current = null;
      }
      stopCapture();
    };
  }, [isRecording, distinguishLeftRight, onRecorded, stopCapture]);

  return {
    isRecording,
    recordedShortcut,
    permissionError,
    startRecording,
    stopRecording,
    clearRecordedShortcut,
    openAccessibilityPreferences,
  };
}
