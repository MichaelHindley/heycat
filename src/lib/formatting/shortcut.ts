/**
 * Keyboard shortcut formatting utilities.
 */

import { mediaKeyMap, keyMap, type CapturedKeyEvent } from "../constants/keyMappings";

/**
 * Format modifier with optional left/right distinction.
 */
function formatModifier(
  isPressed: boolean,
  isLeft: boolean,
  isRight: boolean,
  symbol: string,
  distinguishLeftRight: boolean
): string {
  if (!isPressed) return "";
  if (!distinguishLeftRight) return symbol;
  if (isLeft && !isRight) return `L${symbol}`;
  if (isRight && !isLeft) return `R${symbol}`;
  return symbol; // Both or neither - just show symbol
}

/**
 * Check if a key name is a modifier key.
 */
function isModifierKeyName(keyName: string): boolean {
  return ["Command", "Control", "Alt", "Shift", "fn"].includes(keyName);
}

/**
 * Convert backend key event to display string (e.g., "fn⌘⇧R").
 *
 * @param event - Captured key event from the backend
 * @param distinguishLeftRight - Whether to show L/R prefix for modifiers
 * @returns Human-readable shortcut string with symbols
 */
export function formatBackendKeyForDisplay(
  event: CapturedKeyEvent,
  distinguishLeftRight: boolean = false
): string {
  const parts: string[] = [];
  const isModifierKey = isModifierKeyName(event.key_name);
  const isModifierRelease = isModifierKey && !event.pressed;

  // For modifier release events, the released key's flag is already false
  // So we need to infer it from key_name
  const fnPressed = event.fn_key || (isModifierRelease && event.key_name === "fn");
  const cmdPressed = event.command || (isModifierRelease && event.key_name === "Command");
  const ctrlPressed = event.control || (isModifierRelease && event.key_name === "Control");
  const altPressed = event.alt || (isModifierRelease && event.key_name === "Alt");
  const shiftPressed = event.shift || (isModifierRelease && event.key_name === "Shift");

  // Add modifiers in standard order (fn first since it's special)
  if (fnPressed) parts.push("fn");

  const cmdDisplay = formatModifier(
    cmdPressed,
    event.command_left,
    event.command_right,
    "⌘",
    distinguishLeftRight
  );
  if (cmdDisplay) parts.push(cmdDisplay);

  const ctrlDisplay = formatModifier(
    ctrlPressed,
    event.control_left,
    event.control_right,
    "⌃",
    distinguishLeftRight
  );
  if (ctrlDisplay) parts.push(ctrlDisplay);

  const altDisplay = formatModifier(
    altPressed,
    event.alt_left,
    event.alt_right,
    "⌥",
    distinguishLeftRight
  );
  if (altDisplay) parts.push(altDisplay);

  const shiftDisplay = formatModifier(
    shiftPressed,
    event.shift_left,
    event.shift_right,
    "⇧",
    distinguishLeftRight
  );
  if (shiftDisplay) parts.push(shiftDisplay);

  // Add the main key (excluding modifier keys themselves)
  if (!isModifierKey) {
    // Check if it's a media key first
    if (event.is_media_key && mediaKeyMap[event.key_name]) {
      parts.push(mediaKeyMap[event.key_name]);
    } else {
      parts.push(keyMap[event.key_name] || event.key_name);
    }
  }

  return parts.join("");
}

/**
 * Convert backend key event to backend format (e.g., "Function+Command+Shift+R").
 *
 * @param event - Captured key event from the backend
 * @returns Backend-compatible shortcut string with modifier names
 */
export function formatBackendKeyForBackend(event: CapturedKeyEvent): string {
  const parts: string[] = [];
  const isModifierKey = isModifierKeyName(event.key_name);
  const isModifierRelease = isModifierKey && !event.pressed;

  // For modifier release events, the released key's flag is already false
  // So we need to infer it from key_name
  const fnPressed = event.fn_key || (isModifierRelease && event.key_name === "fn");
  const cmdPressed = event.command || (isModifierRelease && event.key_name === "Command");
  const ctrlPressed = event.control || (isModifierRelease && event.key_name === "Control");
  const altPressed = event.alt || (isModifierRelease && event.key_name === "Alt");
  const shiftPressed = event.shift || (isModifierRelease && event.key_name === "Shift");

  // Add modifiers in standard order
  // Note: Tauri's global-shortcut uses "Function" for fn key
  if (fnPressed) parts.push("Function");
  if (cmdPressed) parts.push("Command");
  if (ctrlPressed) parts.push("Control");
  if (altPressed) parts.push("Alt");
  if (shiftPressed) parts.push("Shift");

  // Add the main key (excluding modifier keys themselves)
  if (!isModifierKey) {
    parts.push(event.key_name);
  }

  return parts.join("+");
}

/**
 * Check if a captured key event represents a valid hotkey.
 *
 * @param event - Captured key event from the backend
 * @returns True if the event is a valid hotkey
 */
export function isValidHotkey(event: CapturedKeyEvent): boolean {
  const isModifierKey = isModifierKeyName(event.key_name);

  // Don't accept modifier-only key PRESSES - this prevents capturing Cmd before Cmd+A
  // Modifier-only shortcuts are captured on RELEASE (pressed=false)
  if (isModifierKey && event.pressed) return false;

  // For modifier-only shortcuts: accept on key release
  // The released key itself won't show in the flags anymore, so we accept any modifier release
  if (isModifierKey && !event.pressed) {
    return true; // Modifier key was released - this is a valid modifier-only shortcut
  }

  const hasMainKey = !isModifierKey;
  const isMediaKey = event.is_media_key;

  // Valid if: has a main key (with or without modifiers), OR is a media key
  return hasMainKey || isMediaKey;
}
