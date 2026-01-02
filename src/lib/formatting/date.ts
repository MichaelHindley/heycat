/**
 * Date formatting utilities.
 */

/**
 * Formats an ISO date string to a localized short date format.
 * Example: "2024-01-15T10:30:00Z" -> "Jan 15, 2024"
 *
 * @param isoString - ISO 8601 date string
 * @returns Formatted date string
 */
export function formatDate(isoString: string): string {
  const date = new Date(isoString);
  return date.toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

/**
 * Formats a Date object to a localized short date format.
 *
 * @param date - Date object
 * @returns Formatted date string
 */
export function formatDateFromDate(date: Date): string {
  return date.toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}
