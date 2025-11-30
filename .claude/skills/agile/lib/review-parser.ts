import type { ReviewVerdict } from "./types";

// ============================================================================
// Parsed Review Types
// ============================================================================

export interface CriterionResult {
  criterion: string;
  status: "PASS" | "FAIL" | "DEFERRED";
  evidence: string;
}

export interface TestCoverageResult {
  testCase: string;
  status: "PASS" | "MISSING";
  location: string;
}

export interface ParsedReview {
  reviewedDate: string;
  reviewer: string;
  acceptanceCriteria: CriterionResult[];
  testCoverage: TestCoverageResult[];
  strengths: string[];
  concerns: string[];
  verdict: ReviewVerdict | null;
  verdictSummary: string;
}

export interface FailedItems {
  criteria: CriterionResult[];
  tests: TestCoverageResult[];
  concerns: string[];
}

// ============================================================================
// Parser Functions
// ============================================================================

/**
 * Parse the ## Review section from spec content
 * Returns null if no review section exists
 */
export function parseReviewSection(specContent: string): ParsedReview | null {
  // Find Review section - everything after ## Review until next ## (not ###) or end
  // First try to find ## Review and capture until the next ## that is NOT ###
  let reviewContent: string | null = null;

  // Check if there's another ## section after ## Review (that's not ###)
  const hasNextSection = /^## Review\n[\s\S]*?\n## (?!#)/m.test(specContent);

  if (hasNextSection) {
    // Use lazy match to stop at next ## section
    const match = specContent.match(/^## Review\n([\s\S]*?)(?=\n## (?!#))/m);
    if (match) reviewContent = match[1];
  } else {
    // ## Review is the last section, capture everything until end
    const match = specContent.match(/^## Review\n([\s\S]+)$/m);
    if (match) reviewContent = match[1];
  }

  if (!reviewContent) return null;

  // Parse metadata
  const dateMatch = reviewContent.match(/\*\*Reviewed:\*\*\s*(\d{4}-\d{2}-\d{2})/);
  const reviewerMatch = reviewContent.match(/\*\*Reviewer:\*\*\s*(.+)/);

  // Parse Acceptance Criteria table
  const acceptanceCriteria = parseAcceptanceCriteriaTable(reviewContent);

  // Parse Test Coverage table
  const testCoverage = parseTestCoverageTable(reviewContent);

  // Parse Code Quality section
  const { strengths, concerns } = parseCodeQuality(reviewContent);

  // Parse Verdict
  const { verdict, summary } = parseVerdict(reviewContent);

  return {
    reviewedDate: dateMatch?.[1] || "",
    reviewer: reviewerMatch?.[1]?.trim() || "",
    acceptanceCriteria,
    testCoverage,
    strengths,
    concerns,
    verdict,
    verdictSummary: summary,
  };
}

/**
 * Check if spec content has a review section
 */
export function hasReviewSection(specContent: string): boolean {
  return /^## Review\n/m.test(specContent);
}

/**
 * Get the verdict from a review section
 */
export function getReviewVerdict(specContent: string): ReviewVerdict | null {
  const review = parseReviewSection(specContent);
  return review?.verdict || null;
}

/**
 * Extract all FAIL/MISSING items from a parsed review
 */
export function getFailedItems(review: ParsedReview): FailedItems {
  return {
    criteria: review.acceptanceCriteria.filter((c) => c.status === "FAIL"),
    tests: review.testCoverage.filter((t) => t.status === "MISSING"),
    concerns: review.concerns,
  };
}

// ============================================================================
// Private Parsing Helpers
// ============================================================================

function parseAcceptanceCriteriaTable(content: string): CriterionResult[] {
  const section = extractSection(content, "Acceptance Criteria Verification");
  if (!section) return [];

  return parseMarkdownTable(section, (cells) => {
    if (cells.length < 3) return null;
    const status = cells[1].trim().toUpperCase();
    if (!["PASS", "FAIL", "DEFERRED"].includes(status)) return null;
    return {
      criterion: cells[0].trim(),
      status: status as "PASS" | "FAIL" | "DEFERRED",
      evidence: cells[2].trim(),
    };
  });
}

function parseTestCoverageTable(content: string): TestCoverageResult[] {
  const section = extractSection(content, "Test Coverage Audit");
  if (!section) return [];

  return parseMarkdownTable(section, (cells) => {
    if (cells.length < 3) return null;
    const status = cells[1].trim().toUpperCase();
    if (!["PASS", "MISSING"].includes(status)) return null;
    return {
      testCase: cells[0].trim(),
      status: status as "PASS" | "MISSING",
      location: cells[2].trim(),
    };
  });
}

function parseCodeQuality(content: string): { strengths: string[]; concerns: string[] } {
  const section = extractSection(content, "Code Quality");
  if (!section) return { strengths: [], concerns: [] };

  const strengths = extractBulletList(section, "Strengths:");
  const concerns = extractBulletList(section, "Concerns:");

  // Filter out "None identified" from concerns
  const filteredConcerns = concerns.filter(
    (c) => !c.toLowerCase().includes("none identified")
  );

  return { strengths, concerns: filteredConcerns };
}

function parseVerdict(content: string): { verdict: ReviewVerdict | null; summary: string } {
  const section = extractSection(content, "Verdict");
  if (!section) return { verdict: null, summary: "" };

  // Match **APPROVED** or **NEEDS_WORK** or just APPROVED/NEEDS_WORK
  const verdictMatch = section.match(/\*?\*?(APPROVED|NEEDS_WORK)\*?\*?/i);
  if (!verdictMatch) return { verdict: null, summary: "" };

  const verdict = verdictMatch[1].toUpperCase() as ReviewVerdict;

  // Extract summary - everything after the verdict and separator
  const summaryMatch = section.match(/\*?\*?(?:APPROVED|NEEDS_WORK)\*?\*?\s*[-–—]\s*(.+)/i);
  const summary = summaryMatch?.[1]?.trim() || "";

  return { verdict, summary };
}

function extractSection(content: string, sectionName: string): string | null {
  // Match ### Section Name followed by content until next ### or end
  // Check if there's another ### section after this one
  const escapedName = escapeRegex(sectionName);
  const hasNextSection = new RegExp(`### ${escapedName}\\n[\\s\\S]*?\\n### `, "m").test(content);

  let match: RegExpMatchArray | null;
  if (hasNextSection) {
    // Stop at next ### section
    const regex = new RegExp(`### ${escapedName}\\n([\\s\\S]*?)(?=\\n### )`, "m");
    match = content.match(regex);
  } else {
    // This is the last ### section, capture until end
    const regex = new RegExp(`### ${escapedName}\\n([\\s\\S]+)$`, "m");
    match = content.match(regex);
  }
  return match?.[1]?.trim() || null;
}

function parseMarkdownTable<T>(content: string, rowMapper: (cells: string[]) => T | null): T[] {
  const results: T[] = [];
  const lines = content.split("\n");

  let headerFound = false;
  for (const line of lines) {
    // Skip non-table lines
    if (!line.startsWith("|")) continue;

    // Skip separator line (contains ---)
    if (line.includes("---")) {
      headerFound = true;
      continue;
    }

    // Skip header row (first | row before ---)
    if (!headerFound) continue;

    // Parse data row
    const cells = line
      .split("|")
      .slice(1, -1) // Remove first and last empty elements from split
      .map((c) => c.trim());

    const result = rowMapper(cells);
    if (result) results.push(result);
  }

  return results;
}

function extractBulletList(content: string, marker: string): string[] {
  const markerIndex = content.indexOf(marker);
  if (markerIndex === -1) return [];

  const afterMarker = content.slice(markerIndex + marker.length);
  const lines = afterMarker.split("\n");
  const items: string[] = [];

  for (const line of lines) {
    const bulletMatch = line.match(/^[-*]\s+(.+)/);
    if (bulletMatch) {
      items.push(bulletMatch[1].trim());
    } else if (line.trim() && !line.startsWith("-") && !line.startsWith("*")) {
      // Non-bullet, non-empty line after we've started - check if it's a new section
      if (line.startsWith("**") || line.startsWith("###")) {
        break;
      }
      // Could be continuation of previous bullet, skip
    }
  }

  return items;
}

function escapeRegex(str: string): string {
  return str.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
