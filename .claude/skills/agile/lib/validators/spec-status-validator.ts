import type { SpecStatus, ReviewVerdict } from "../types";

// ============================================================================
// Valid Spec Status Transitions
// ============================================================================

export const VALID_SPEC_TRANSITIONS: Record<SpecStatus, SpecStatus[]> = {
  pending: ["in-progress"],
  "in-progress": ["pending", "in-review"],
  "in-review": ["in-progress", "completed"],
  completed: ["in-review"],
};

// ============================================================================
// Validation Types
// ============================================================================

export interface SpecTransitionContext {
  currentStatus: SpecStatus;
  targetStatus: SpecStatus;
  hasReviewSection: boolean;
  reviewVerdict?: ReviewVerdict | null;
}

export interface SpecTransitionResult {
  valid: boolean;
  error?: string;
}

// ============================================================================
// Validation Function
// ============================================================================

/**
 * Validate a spec status transition
 *
 * Rules:
 * - pending → in-progress: Always allowed
 * - in-progress → pending: Always allowed (undo)
 * - in-progress → in-review: Always allowed (ready for review)
 * - in-review → completed: Requires APPROVED verdict
 * - in-review → in-progress: Requires NEEDS_WORK verdict
 * - completed → in-review: Always allowed (re-review)
 */
export function validateSpecTransition(context: SpecTransitionContext): SpecTransitionResult {
  const { currentStatus, targetStatus, hasReviewSection, reviewVerdict } = context;

  // Check if transition is allowed at all
  const allowedTransitions = VALID_SPEC_TRANSITIONS[currentStatus];
  if (!allowedTransitions.includes(targetStatus)) {
    return {
      valid: false,
      error: `Cannot transition from "${currentStatus}" to "${targetStatus}". ` +
        `Allowed transitions: ${allowedTransitions.join(", ") || "none"}`,
    };
  }

  // Special validation rules for specific transitions
  if (currentStatus === "in-review" && targetStatus === "completed") {
    // Requires review with APPROVED verdict
    if (!hasReviewSection) {
      return {
        valid: false,
        error: "Cannot complete: spec must have a Review section",
      };
    }
    if (reviewVerdict !== "APPROVED") {
      return {
        valid: false,
        error: `Cannot complete: review verdict must be APPROVED (current: ${reviewVerdict || "none"})`,
      };
    }
  }

  if (currentStatus === "in-review" && targetStatus === "in-progress") {
    // Requires review with NEEDS_WORK verdict
    if (!hasReviewSection) {
      return {
        valid: false,
        error: "Cannot return to in-progress: spec must have a Review section",
      };
    }
    if (reviewVerdict !== "NEEDS_WORK") {
      return {
        valid: false,
        error: `Cannot return to in-progress: review verdict must be NEEDS_WORK (current: ${reviewVerdict || "none"})`,
      };
    }
  }

  return { valid: true };
}

/**
 * Check if a status is the terminal state for a spec
 */
export function isTerminalStatus(status: SpecStatus): boolean {
  return status === "completed";
}

/**
 * Check if a spec needs review based on its status
 */
export function needsReview(status: SpecStatus): boolean {
  return status === "in-review";
}
