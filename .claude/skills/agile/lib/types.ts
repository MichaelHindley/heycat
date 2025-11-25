// ============================================================================
// Stage Types and Constants
// ============================================================================

export const STAGES = ["1-backlog", "2-todo", "3-in-progress", "4-review", "5-done"] as const;
export type Stage = (typeof STAGES)[number];

export const STAGE_NAMES: Record<Stage, string> = {
  "1-backlog": "Backlog",
  "2-todo": "Todo",
  "3-in-progress": "In Progress",
  "4-review": "Review",
  "5-done": "Done",
};

// Strict sequential transitions only
export const VALID_TRANSITIONS: Record<Stage, Stage[]> = {
  "1-backlog": ["2-todo"],
  "2-todo": ["1-backlog", "3-in-progress"],
  "3-in-progress": ["2-todo", "4-review"],
  "4-review": ["3-in-progress", "5-done"],
  "5-done": ["4-review"],
};

// ============================================================================
// Template Types and Constants
// ============================================================================

export const TEMPLATES = ["feature", "bug", "task"] as const;
export type Template = (typeof TEMPLATES)[number];

// ============================================================================
// Path Constants
// ============================================================================

export const AGILE_DIR = "agile";
export const ARCHIVE_DIR = "agile/archive";
export const TEMPLATES_DIR = "agile/templates";

export const SLUG_PATTERN = /^[a-z0-9]+(-[a-z0-9]+)*$/;

// ============================================================================
// Interfaces
// ============================================================================

export interface IssueLocation {
  stage: Stage;
  path: string;
  filename: string;
}

export interface IssueMeta {
  title: string;
  type: Template | "unknown";
  created: string;
  owner: string;
}
