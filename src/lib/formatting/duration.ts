/**
 * Duration formatting utilities.
 */

/**
 * Formats a duration in seconds to MM:SS format.
 * Example: 125 -> "2:05"
 *
 * @param seconds - Duration in seconds
 * @returns Formatted duration string
 */
export function formatDuration(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}

/**
 * Formats a duration in seconds to a human-readable string.
 * Example: 125 -> "2 min 5 sec"
 *
 * @param seconds - Duration in seconds
 * @returns Human-readable duration string
 */
export function formatDurationLong(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);

  if (mins === 0) {
    return `${secs} sec`;
  }
  if (secs === 0) {
    return `${mins} min`;
  }
  return `${mins} min ${secs} sec`;
}
