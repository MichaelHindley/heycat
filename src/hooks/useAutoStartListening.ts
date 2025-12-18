import { useEffect, useRef } from "react";
import { load } from "@tauri-apps/plugin-store";
import { invoke } from "@tauri-apps/api/core";

const STORE_FILE = "settings.json";

/**
 * Hook that checks settings on mount and auto-enables listening if configured.
 * Runs once at app startup to honor the autoStartOnLaunch preference.
 */
export function useAutoStartListening(): void {
  const hasChecked = useRef(false);

  useEffect(() => {
    // Only run once per app session
    if (hasChecked.current) return;
    hasChecked.current = true;

    /* v8 ignore start -- @preserve */
    const checkAndAutoStart = async () => {
      try {
        const store = await load(STORE_FILE);
        const autoStartOnLaunch = await store.get<boolean>(
          "listening.autoStartOnLaunch"
        );

        if (autoStartOnLaunch) {
          // Get selected device from settings
          const selectedDevice = await store.get<string | null>(
            "audio.selectedDevice"
          );
          await invoke("enable_listening", {
            deviceName: selectedDevice ?? undefined,
          });
        }
      } catch {
        // Silently ignore errors - this is a best-effort feature
      }
    };

    checkAndAutoStart();
    /* v8 ignore stop */
  }, []);
}
