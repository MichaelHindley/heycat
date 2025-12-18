import { useEffect } from "react";
import { useUIMode, type UIMode } from "../../hooks/useUIMode";

export interface UIToggleProps {
  /** Override the mode externally (for testing) */
  mode?: UIMode;
  /** Override the toggle function externally (for testing) */
  onToggle?: () => void;
}

/**
 * Development-only toggle to switch between old and new UI
 * Fixed position in bottom-left corner, above footer
 * Only renders in development mode
 */
export function UIToggle({ mode: externalMode, onToggle }: UIToggleProps = {}) {
  const { mode: internalMode, toggle: internalToggle } = useUIMode();

  const mode = externalMode ?? internalMode;
  const toggle = onToggle ?? internalToggle;

  // Keyboard shortcut: Ctrl+Shift+U
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.shiftKey && e.key === "U") {
        e.preventDefault();
        toggle();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [toggle]);

  // Only render in development mode
  if (!import.meta.env.DEV) {
    return null;
  }

  return (
    <div
      className="fixed bottom-4 left-4 z-40 flex items-center gap-2"
      data-testid="ui-toggle"
    >
      <button
        onClick={toggle}
        className="
          flex items-center gap-2
          px-3 py-1.5
          rounded-full
          bg-surface-secondary/90
          backdrop-blur-sm
          border border-border-subtle
          shadow-md
          text-xs font-medium
          text-text-secondary
          hover:bg-surface-tertiary
          hover:text-text-primary
          transition-colors duration-[var(--duration-fast)]
          focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-heycat-teal focus-visible:ring-offset-2
        "
        aria-label={`Switch to ${mode === "old" ? "new" : "old"} UI`}
        title="Toggle UI mode (Ctrl+Shift+U)"
      >
        <span
          className={`
            w-2 h-2 rounded-full
            ${mode === "new" ? "bg-heycat-teal" : "bg-heycat-orange"}
          `}
        />
        <span>{mode === "new" ? "New UI" : "Old UI"}</span>
      </button>
    </div>
  );
}
