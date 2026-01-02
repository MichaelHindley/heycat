/**
 * File size formatting utilities.
 */

/**
 * Formats a file size in bytes to a human-readable string.
 * Example: 1536 -> "1.5 KB"
 *
 * @param bytes - File size in bytes
 * @returns Formatted file size string
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 B";

  const units = ["B", "KB", "MB", "GB"];
  const k = 1024;
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  const value = bytes / Math.pow(k, i);

  // No decimal for bytes, one decimal for larger units
  return `${value.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}
