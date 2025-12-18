import { Cat, Settings, HelpCircle, Command } from "lucide-react";
import { StatusPill, type StatusPillStatus } from "../ui";

export interface HeaderProps {
  /** Current status for the status pill */
  status?: StatusPillStatus;
  /** Status label override */
  statusLabel?: string;
  /** Recording duration in seconds (shown when status is recording) */
  recordingDuration?: number;
  /** Callback when command palette trigger is clicked */
  onCommandPaletteOpen?: () => void;
  /** Callback when settings is clicked */
  onSettingsClick?: () => void;
  /** Callback when help is clicked */
  onHelpClick?: () => void;
}

export function Header({
  status = "idle",
  statusLabel,
  recordingDuration,
  onCommandPaletteOpen,
  onSettingsClick,
  onHelpClick,
}: HeaderProps) {
  return (
    <header
      className="h-12 flex items-center justify-between px-4 border-b border-border bg-surface shrink-0"
      role="banner"
    >
      {/* Left: Logo */}
      <div className="flex items-center gap-2">
        <Cat
          className="w-6 h-6 text-heycat-orange"
          aria-hidden="true"
        />
        <span className="text-lg font-semibold text-text-primary">
          HeyCat
        </span>
      </div>

      {/* Center: Status Pill */}
      <div className="absolute left-1/2 -translate-x-1/2">
        <StatusPill
          status={status}
          label={statusLabel}
          recordingDuration={recordingDuration}
        />
      </div>

      {/* Right: Actions */}
      <div className="flex items-center gap-1">
        {/* Command Palette Trigger */}
        <button
          type="button"
          onClick={onCommandPaletteOpen}
          className="
            flex items-center gap-1.5 px-2 py-1
            text-sm text-text-secondary
            bg-text-secondary/10 hover:bg-text-secondary/20
            rounded-[var(--radius-sm)]
            transition-colors duration-[var(--duration-fast)]
          "
          aria-label="Open command palette (Command K)"
        >
          <Command className="w-3.5 h-3.5" aria-hidden="true" />
          <span>K</span>
        </button>

        {/* Settings */}
        <button
          type="button"
          onClick={onSettingsClick}
          className="
            p-2
            text-text-secondary hover:text-text-primary
            hover:bg-text-secondary/10
            rounded-[var(--radius-sm)]
            transition-colors duration-[var(--duration-fast)]
          "
          aria-label="Settings"
        >
          <Settings className="w-5 h-5" aria-hidden="true" />
        </button>

        {/* Help */}
        <button
          type="button"
          onClick={onHelpClick}
          className="
            p-2
            text-text-secondary hover:text-text-primary
            hover:bg-text-secondary/10
            rounded-[var(--radius-sm)]
            transition-colors duration-[var(--duration-fast)]
          "
          aria-label="Help"
        >
          <HelpCircle className="w-5 h-5" aria-hidden="true" />
        </button>
      </div>
    </header>
  );
}
