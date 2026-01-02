/* v8 ignore file -- @preserve */
// Frontend logging utilities using Tauri logging plugin

import { attachConsole } from "@tauri-apps/plugin-log";

// Re-export log functions for convenient importing
export { trace, debug, info, warn, error } from "@tauri-apps/plugin-log";

let detachFn: (() => void) | null = null;

/**
 * Initialize the logger by attaching to the console.
 * This forwards Rust backend logs to the browser DevTools console.
 * Call once on app startup.
 */
export async function initLogger(): Promise<void> {
  if (detachFn) return; // Already initialized

  try {
    detachFn = await attachConsole();
  } catch (err) {
    console.error("[heycat] Failed to initialize logger:", err);
  }
}

/**
 * Cleanup the logger by detaching from console.
 * Call on app shutdown if needed.
 */
export function cleanupLogger(): void {
  if (detachFn) {
    detachFn();
    detachFn = null;
  }
}
