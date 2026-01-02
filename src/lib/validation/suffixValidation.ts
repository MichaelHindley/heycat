/**
 * Suffix validation utilities for dictionary entries.
 */

/** Maximum allowed length for suffix characters */
export const MAX_SUFFIX_LENGTH = 5;

/**
 * Validates that a suffix is within the allowed length.
 * @param suffix - The suffix to validate
 * @returns Error message if invalid, null if valid
 */
export function validateSuffix(suffix: string): string | null {
  if (suffix.length > MAX_SUFFIX_LENGTH) {
    return `Suffix must be ${MAX_SUFFIX_LENGTH} characters or less`;
  }
  return null;
}

/**
 * Checks if a suffix exceeds the maximum allowed length.
 * @param suffix - The suffix to check
 * @returns True if suffix is too long
 */
export function isSuffixTooLong(suffix: string): boolean {
  return suffix.length > MAX_SUFFIX_LENGTH;
}
