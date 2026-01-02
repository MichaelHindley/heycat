/**
 * Keyboard key mappings for display and backend integration.
 */

/**
 * Backend captured key event structure from CGEventTap.
 * Includes left/right modifier distinction and media key support.
 */
export interface CapturedKeyEvent {
  key_code: number;
  key_name: string;
  fn_key: boolean;
  command: boolean;
  command_left: boolean;
  command_right: boolean;
  control: boolean;
  control_left: boolean;
  control_right: boolean;
  alt: boolean;
  alt_left: boolean;
  alt_right: boolean;
  shift: boolean;
  shift_left: boolean;
  shift_right: boolean;
  pressed: boolean;
  is_media_key: boolean;
}

/**
 * Media key display mapping.
 * Maps backend media key names to display symbols/emoji.
 */
export const mediaKeyMap: Record<string, string> = {
  VolumeUp: "🔊",
  VolumeDown: "🔉",
  Mute: "🔇",
  BrightnessUp: "🔆",
  BrightnessDown: "🔅",
  PlayPause: "⏯",
  NextTrack: "⏭",
  PreviousTrack: "⏮",
  FastForward: "⏩",
  Rewind: "⏪",
  KeyboardBrightnessUp: "🔆⌨",
  KeyboardBrightnessDown: "🔅⌨",
};

/**
 * Special key display mapping.
 * Maps backend key names to display symbols.
 */
export const keyMap: Record<string, string> = {
  Up: "↑",
  Down: "↓",
  Left: "←",
  Right: "→",
  Enter: "↵",
  Backspace: "⌫",
  Delete: "⌦",
  Escape: "Esc",
  Tab: "⇥",
  Space: "Space",
};

/**
 * Modifier key symbols for macOS display.
 */
export const modifierSymbols = {
  command: "⌘",
  control: "⌃",
  alt: "⌥",
  shift: "⇧",
  fn: "fn",
} as const;
