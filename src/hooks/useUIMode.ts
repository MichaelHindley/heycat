import { useState, useEffect, useCallback } from "react";

const UI_MODE_KEY = "heycat-ui-mode";

export type UIMode = "old" | "new";

export interface UseUIModeReturn {
  mode: UIMode;
  setMode: (mode: UIMode) => void;
  toggle: () => void;
}

/**
 * Hook to manage UI mode state (old vs new UI)
 * Persists preference to localStorage
 * Only active in development mode
 */
export function useUIMode(): UseUIModeReturn {
  const [mode, setModeState] = useState<UIMode>(() => {
    if (typeof window === "undefined") return "old";
    const stored = localStorage.getItem(UI_MODE_KEY);
    return stored === "new" ? "new" : "old";
  });

  // Persist to localStorage when mode changes
  useEffect(() => {
    localStorage.setItem(UI_MODE_KEY, mode);
  }, [mode]);

  const setMode = useCallback((newMode: UIMode) => {
    setModeState(newMode);
  }, []);

  const toggle = useCallback(() => {
    setModeState((prev) => (prev === "old" ? "new" : "old"));
  }, []);

  return { mode, setMode, toggle };
}
