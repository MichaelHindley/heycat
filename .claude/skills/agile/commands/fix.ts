import { readFile } from "node:fs/promises";
import { findProjectRoot, getCurrentDate } from "../lib/utils";
import { createIssueResolver } from "../lib/issue-resolver";
import { createSpecManager } from "../lib/spec-manager";
import { parseReviewSection, getFailedItems, type ParsedReview } from "../lib/review-parser";
import type { Issue, SpecInfo, ReviewHistoryEntry } from "../lib/types";

export async function handleFix(args: string[]): Promise<void> {
  const projectRoot = await findProjectRoot();
  const resolver = createIssueResolver();
  const specManager = createSpecManager();

  // Find in-progress issues
  const inProgressIssues = await resolver.listIssues(projectRoot, "3-in-progress");

  if (inProgressIssues.length === 0) {
    console.error("No issues found in 3-in-progress stage.");
    console.error("Move an issue to in-progress first: agile.ts move <issue> 3-in-progress");
    process.exit(1);
  }

  if (inProgressIssues.length > 1) {
    console.error("Multiple issues in 3-in-progress:");
    for (const issue of inProgressIssues) {
      console.error(`  - ${issue.name}`);
    }
    console.error("\nThis command currently only supports a single in-progress issue.");
    process.exit(1);
  }

  const issue = inProgressIssues[0];

  // Find specs with in-review status and NEEDS_WORK verdict
  const specs = await specManager.listSpecs(issue);
  const inReviewSpecs = specs.filter((s) => s.frontmatter.status === "in-review");

  if (inReviewSpecs.length === 0) {
    console.error(`No specs in "in-review" status found in issue "${issue.name}".`);
    console.error("Run /agile:review first to review a spec.");
    process.exit(1);
  }

  // Find specs with NEEDS_WORK verdict
  const specsWithNeedsWork: Array<{ spec: SpecInfo; review: ParsedReview }> = [];

  for (const spec of inReviewSpecs) {
    const content = await readFile(spec.path, "utf-8");
    const review = parseReviewSection(content);
    if (review && review.verdict === "NEEDS_WORK") {
      specsWithNeedsWork.push({ spec, review });
    }
  }

  if (specsWithNeedsWork.length === 0) {
    console.error(`No specs with NEEDS_WORK verdict found in issue "${issue.name}".`);
    console.error("All in-review specs have either no review or APPROVED verdict.");
    process.exit(1);
  }

  // Use the first spec with NEEDS_WORK (could be enhanced to allow selection)
  const { spec, review } = specsWithNeedsWork[0];

  // Get failed items
  const failedItems = getFailedItems(review);

  // Output the fix guide
  outputFixGuide(issue, spec, review, failedItems);

  // Transition spec back to in-progress
  console.log("\n" + "=".repeat(80));
  console.log("TRANSITIONING SPEC TO IN-PROGRESS...");
  console.log("=".repeat(80));

  try {
    // Add review history entry before transitioning
    const historyEntry: ReviewHistoryEntry = {
      round: spec.frontmatter.review_round || 1,
      date: review.reviewedDate || getCurrentDate(),
      verdict: "NEEDS_WORK",
      failedCriteria: failedItems.criteria.map((c) => c.criterion),
      concerns: failedItems.concerns,
    };

    await specManager.addReviewHistoryEntry(spec, historyEntry);
    await specManager.updateStatus(spec, "in-progress");

    console.log(`Spec "${spec.name}" transitioned to in-progress.`);
    console.log("\nAfter making fixes, run: /agile:review");
  } catch (err) {
    console.error(`Failed to transition spec: ${err instanceof Error ? err.message : err}`);
    process.exit(1);
  }
}

function outputFixGuide(
  issue: Issue,
  spec: SpecInfo,
  review: ParsedReview,
  failedItems: ReturnType<typeof getFailedItems>
): void {
  console.log("=".repeat(80));
  console.log(`FIX GUIDE: ${spec.name}`);
  console.log("=".repeat(80));
  console.log();
  console.log(`Issue: ${issue.name} (3-in-progress)`);
  console.log(`Spec: ${spec.name}.spec.md`);
  console.log(`Review Round: ${spec.frontmatter.review_round || 1}`);
  console.log(`Verdict: NEEDS_WORK`);
  console.log(`Summary: ${review.verdictSummary}`);
  console.log();

  // Failed Acceptance Criteria
  if (failedItems.criteria.length > 0) {
    console.log("-".repeat(80));
    console.log(`FAILED ACCEPTANCE CRITERIA (${failedItems.criteria.length} items):`);
    console.log("-".repeat(80));
    console.log();

    failedItems.criteria.forEach((item, index) => {
      console.log(`${index + 1}. [FAIL] ${item.criterion}`);
      console.log(`   Evidence: ${item.evidence}`);
      console.log();
    });
  }

  // Missing Test Coverage
  if (failedItems.tests.length > 0) {
    console.log("-".repeat(80));
    console.log(`MISSING TEST COVERAGE (${failedItems.tests.length} items):`);
    console.log("-".repeat(80));
    console.log();

    failedItems.tests.forEach((item, index) => {
      console.log(`${index + 1}. [MISSING] ${item.testCase}`);
      console.log(`   Expected: ${item.location}`);
      console.log();
    });
  }

  // Code Quality Concerns
  if (failedItems.concerns.length > 0) {
    console.log("-".repeat(80));
    console.log(`CODE QUALITY CONCERNS (${failedItems.concerns.length} items):`);
    console.log("-".repeat(80));
    console.log();

    failedItems.concerns.forEach((concern, index) => {
      console.log(`${index + 1}. ${concern}`);
    });
    console.log();
  }

  // No issues found (shouldn't happen with NEEDS_WORK but handle gracefully)
  if (failedItems.criteria.length === 0 && failedItems.tests.length === 0 && failedItems.concerns.length === 0) {
    console.log("-".repeat(80));
    console.log("NO SPECIFIC ISSUES IDENTIFIED");
    console.log("-".repeat(80));
    console.log();
    console.log("The review has NEEDS_WORK verdict but no specific failures were parsed.");
    console.log("Please review the spec file manually for reviewer feedback.");
    console.log();
  }

  // Actions
  console.log("-".repeat(80));
  console.log("FIX ACTIONS:");
  console.log("-".repeat(80));
  console.log();
  console.log("1. Address each FAIL criterion by fixing the referenced code");
  console.log("2. Add missing tests in the locations noted");
  console.log("3. Address each concern in the Code Quality section");
  console.log("4. Run: tcr check");
  console.log("5. Request re-review: /agile:review");
  console.log();
}
