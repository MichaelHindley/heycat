import { readFile } from "node:fs/promises";
import { type Stage, type Template, STAGE_NAMES } from "./types";

// ============================================================================
// Placeholder Detection
// ============================================================================

const PLACEHOLDER_PATTERNS = [
  /\[[\w\s\-.,!?]+\]/g, // [placeholder text]
  /\[e\.g\.,?\s*[^\]]+\]/gi, // [e.g., examples]
];

// Patterns that look like placeholders but aren't (e.g., checkboxes)
const CHECKBOX_PATTERN = /\[\s*[xX]?\s*\]/g;

/**
 * Check if a string contains placeholder text (excluding checkboxes)
 */
export function hasPlaceholders(text: string): boolean {
  // Remove checkboxes before checking for placeholders
  const textWithoutCheckboxes = text.replace(CHECKBOX_PATTERN, "");
  // Create new RegExp instances to avoid stateful global flag issues
  return PLACEHOLDER_PATTERNS.some((pattern) => {
    const freshPattern = new RegExp(pattern.source, pattern.flags);
    return freshPattern.test(textWithoutCheckboxes);
  });
}

/**
 * Find all placeholders in text (excluding checkboxes)
 */
export function findPlaceholders(text: string): string[] {
  // Remove checkboxes before searching for placeholders
  const textWithoutCheckboxes = text.replace(CHECKBOX_PATTERN, "");
  const placeholders: string[] = [];
  for (const pattern of PLACEHOLDER_PATTERNS) {
    const matches = textWithoutCheckboxes.match(pattern);
    if (matches) {
      placeholders.push(...matches);
    }
  }
  return [...new Set(placeholders)];
}

// ============================================================================
// Section Parsing
// ============================================================================

interface Section {
  name: string;
  content: string;
  hasPlaceholders: boolean;
}

/**
 * Parse markdown content into sections
 */
export function parseSections(content: string): Section[] {
  const sections: Section[] = [];
  const lines = content.split("\n");

  let currentSection: Section | null = null;
  let currentContent: string[] = [];

  for (const line of lines) {
    const headerMatch = line.match(/^##\s+(.+)$/);
    if (headerMatch) {
      // Save previous section
      if (currentSection) {
        currentSection.content = currentContent.join("\n").trim();
        currentSection.hasPlaceholders = hasPlaceholders(currentSection.content);
        sections.push(currentSection);
      }
      // Start new section
      currentSection = {
        name: headerMatch[1],
        content: "",
        hasPlaceholders: false,
      };
      currentContent = [];
    } else if (currentSection) {
      currentContent.push(line);
    }
  }

  // Save last section
  if (currentSection) {
    currentSection.content = currentContent.join("\n").trim();
    currentSection.hasPlaceholders = hasPlaceholders(currentSection.content);
    sections.push(currentSection);
  }

  return sections;
}

/**
 * Find sections that still have placeholder content
 */
export function findIncompleteSections(content: string): string[] {
  const sections = parseSections(content);
  return sections.filter((s) => s.hasPlaceholders).map((s) => s.name);
}

// ============================================================================
// Definition of Done Parsing
// ============================================================================

export interface DoDStatus {
  completed: number;
  total: number;
  items: { text: string; checked: boolean }[];
}

/**
 * Parse Definition of Done checkboxes from issue content
 */
export function parseDoD(content: string): DoDStatus {
  const checkboxPattern = /- \[([ xX])\]\s*(.+)$/gm;
  const items: { text: string; checked: boolean }[] = [];

  let match;
  while ((match = checkboxPattern.exec(content)) !== null) {
    items.push({
      checked: match[1].toLowerCase() === "x",
      text: match[2].trim(),
    });
  }

  return {
    completed: items.filter((i) => i.checked).length,
    total: items.length,
    items,
  };
}

// ============================================================================
// Issue Analysis
// ============================================================================

export interface IssueAnalysis {
  incompleteSections: string[];
  dod: DoDStatus;
  hasDescription: boolean;
  hasTechnicalNotes: boolean;
  ownerAssigned: boolean;
}

/**
 * Analyze issue content for completeness
 */
export function analyzeIssue(content: string, owner: string): IssueAnalysis {
  const sections = parseSections(content);
  const incompleteSections = sections.filter((s) => s.hasPlaceholders).map((s) => s.name);

  const descSection = sections.find((s) => s.name === "Description");
  const techSection = sections.find((s) => s.name === "Technical Notes");

  return {
    incompleteSections,
    dod: parseDoD(content),
    hasDescription: descSection ? !descSection.hasPlaceholders && descSection.content.length > 10 : false,
    hasTechnicalNotes: techSection ? !techSection.hasPlaceholders && techSection.content.length > 10 : false,
    ownerAssigned: owner !== "[Name]" && owner.length > 0,
  };
}

// ============================================================================
// Stage Guidance
// ============================================================================

export interface StageGuidance {
  focus: string;
  actions: string[];
  readinessChecklist: string[];
}

export const STAGE_GUIDANCE: Record<Stage, StageGuidance> = {
  "1-backlog": {
    focus: "Define the issue clearly so it can be prioritized",
    actions: [
      "Populate Description with clear context and purpose",
      "Write acceptance criteria (Given/When/Then for features)",
      "Capture reproduction steps (for bugs)",
      "Document the goal and approach",
    ],
    readinessChecklist: ["Description has no placeholder text", "Basic scope is defined"],
  },
  "2-todo": {
    focus: "Prepare for implementation",
    actions: [
      "Refine acceptance criteria to be testable",
      "Add Technical Notes with implementation approach",
      "Identify dependencies or blockers",
      "Ensure owner is assigned",
    ],
    readinessChecklist: ["Owner is assigned", "Technical Notes present", "No blockers identified"],
  },
  "3-in-progress": {
    focus: "Support active development",
    actions: [
      "Help with implementation questions",
      "Update Technical Notes with discoveries",
      "Track Definition of Done progress",
      "Consider if issue should be split",
    ],
    readinessChecklist: ["Implementation complete", "Ready for review"],
  },
  "4-review": {
    focus: "Ensure quality and completeness",
    actions: [
      "Walk through Definition of Done checklist",
      "Verify acceptance criteria are met",
      "Check tests are passing",
      "Confirm documentation is updated",
    ],
    readinessChecklist: ["All DoD items checked", "All acceptance criteria met"],
  },
  "5-done": {
    focus: "Wrap up and archive",
    actions: ["Celebrate completion!", "Suggest archiving if appropriate", "Identify follow-up work"],
    readinessChecklist: ["Work is complete"],
  },
};

// ============================================================================
// Transition Validation
// ============================================================================

export interface ValidationResult {
  valid: boolean;
  missing: string[];
}

/**
 * Validate if an issue is ready to transition to a new stage
 */
export function validateForTransition(
  toStage: Stage,
  analysis: IssueAnalysis
): ValidationResult {
  const missing: string[] = [];

  switch (toStage) {
    case "2-todo":
      if (!analysis.hasDescription) {
        missing.push("Description section must be complete (no placeholders)");
      }
      break;

    case "3-in-progress":
      if (!analysis.ownerAssigned) {
        missing.push("Owner must be assigned");
      }
      if (!analysis.hasTechnicalNotes) {
        missing.push("Technical Notes section must be present");
      }
      break;

    case "4-review":
      if (analysis.dod.completed === 0 && analysis.dod.total > 0) {
        missing.push("At least one Definition of Done item must be checked");
      }
      break;

    case "5-done":
      if (analysis.dod.completed < analysis.dod.total) {
        missing.push(
          `All Definition of Done items must be checked (${analysis.dod.completed}/${analysis.dod.total})`
        );
      }
      break;
  }

  return {
    valid: missing.length === 0,
    missing,
  };
}

/**
 * Read and analyze an issue file
 */
export async function analyzeIssueFile(
  filePath: string,
  owner: string
): Promise<IssueAnalysis> {
  const content = await readFile(filePath, "utf-8");
  return analyzeIssue(content, owner);
}
