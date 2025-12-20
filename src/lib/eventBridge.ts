/**
 * Central event bridge that routes Tauri backend events to appropriate state managers.
 *
 * Event types and their destinations:
 * - Server state events → Tanstack Query invalidation (triggers refetch)
 * - UI state events → Zustand store updates (direct state mutation)
 *
 * This is the integration layer between backend-initiated events and frontend state.
 * All event subscriptions are set up once on app mount and cleaned up on unmount.
 */
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { QueryClient } from "@tanstack/react-query";
import { queryKeys } from "./queryKeys";
import type { AppState } from "../stores/appStore";

/**
 * Event names emitted by the Rust backend.
 * These must match the constants in src-tauri/src/events.rs
 */
export const eventNames = {
  // Recording events
  RECORDING_STARTED: "recording_started",
  RECORDING_STOPPED: "recording_stopped",
  RECORDING_ERROR: "recording_error",

  // Listening events
  LISTENING_STARTED: "listening_started",
  LISTENING_STOPPED: "listening_stopped",

  // Transcription events
  TRANSCRIPTION_COMPLETED: "transcription_completed",

  // Model events
  MODEL_DOWNLOAD_COMPLETED: "model_download_completed",

  // UI state events
  OVERLAY_MODE: "overlay-mode",
} as const;

/**
 * Payload type for overlay-mode event.
 * The mode can be null to indicate no overlay should be shown.
 */
export type OverlayModePayload = string | null;

/**
 * Sets up the central event bridge that routes Tauri events to state managers.
 *
 * @param queryClient - Tanstack Query client for cache invalidation
 * @param store - Zustand store state for UI updates
 * @returns Cleanup function that unsubscribes all event listeners
 *
 * @example
 * ```typescript
 * const cleanup = await setupEventBridge(queryClient, useAppStore.getState());
 * // On unmount:
 * cleanup();
 * ```
 */
export async function setupEventBridge(
  queryClient: QueryClient,
  store: Pick<AppState, "setOverlayMode">
): Promise<() => void> {
  const unlistenFns: UnlistenFn[] = [];

  // ============================================================
  // Server state events → Query invalidation
  // ============================================================

  // Recording state events - invalidate recording state query
  unlistenFns.push(
    await listen(eventNames.RECORDING_STARTED, () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.tauri.getRecordingState,
      });
    })
  );

  unlistenFns.push(
    await listen(eventNames.RECORDING_STOPPED, () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.tauri.getRecordingState,
      });
    })
  );

  unlistenFns.push(
    await listen(eventNames.RECORDING_ERROR, () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.tauri.getRecordingState,
      });
    })
  );

  // Transcription events - invalidate recordings list
  unlistenFns.push(
    await listen(eventNames.TRANSCRIPTION_COMPLETED, () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.tauri.listRecordings,
      });
    })
  );

  // Listening state events - invalidate listening status query
  unlistenFns.push(
    await listen(eventNames.LISTENING_STARTED, () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.tauri.getListeningStatus,
      });
    })
  );

  unlistenFns.push(
    await listen(eventNames.LISTENING_STOPPED, () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.tauri.getListeningStatus,
      });
    })
  );

  // Model events - invalidate all model status queries
  // Using partial match since model status queries have a type parameter
  unlistenFns.push(
    await listen(eventNames.MODEL_DOWNLOAD_COMPLETED, () => {
      queryClient.invalidateQueries({
        queryKey: ["tauri", "check_parakeet_model_status"],
      });
    })
  );

  // ============================================================
  // UI state events → Zustand updates
  // ============================================================

  // Overlay mode changes - update Zustand store directly
  unlistenFns.push(
    await listen<OverlayModePayload>(eventNames.OVERLAY_MODE, (event) => {
      store.setOverlayMode(event.payload);
    })
  );

  // Return cleanup function that unsubscribes all listeners
  return () => {
    unlistenFns.forEach((unlisten) => unlisten());
  };
}
