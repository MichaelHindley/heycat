import { exec } from "node:child_process";
import { promisify } from "node:util";
import { readFile } from "node:fs/promises";
import { findProjectRoot, getCurrentDate } from "../lib/utils";
import { createIssueResolver } from "../lib/issue-resolver";
import { createSpecManager } from "../lib/spec-manager";
import type { Issue, SpecInfo } from "../lib/types";

const execAsync = promisify(exec);

export async function handleReview(args: string[]): Promise<void> {
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

  // Find specs ready for review (in-review status)
  const specs = await specManager.listSpecs(issue);
  const reviewableSpecs = specs.filter((s) => s.frontmatter.status === "in-review");

  if (reviewableSpecs.length === 0) {
    console.error(`No specs in "in-review" status found in issue "${issue.name}".`);
    console.error("Transition a spec to in-review first: agile.ts spec status <issue> <spec> in-review");
    process.exit(1);
  }

  // Find the most recently modified spec using git
  const latestSpec = await findLatestByGit(reviewableSpecs);

  if (!latestSpec) {
    console.error("Could not determine the latest in-review spec.");
    process.exit(1);
  }

  // Read spec content
  const specContent = await readFile(latestSpec.path, "utf-8");

  // Check if already reviewed
  const hasReview = specContent.includes("## Review");
  if (hasReview) {
    console.log("NOTE: This spec already has a Review section. A new review will replace it.\n");
  }

  // Get git info for the spec
  const lastCommit = await getLastCommitInfo(latestSpec.path);

  // Output the review prompt
  outputReviewPrompt(issue, latestSpec, specContent, lastCommit, hasReview);
}

async function findLatestByGit(specs: SpecInfo[]): Promise<SpecInfo | null> {
  if (specs.length === 0) return null;
  if (specs.length === 1) return specs[0];

  const specsWithDates: Array<{ spec: SpecInfo; date: Date }> = [];

  for (const spec of specs) {
    try {
      const { stdout } = await execAsync(`git log -1 --format=%cI -- "${spec.path}"`);
      const dateStr = stdout.trim();
      if (dateStr) {
        specsWithDates.push({ spec, date: new Date(dateStr) });
      }
    } catch {
      // If git fails, use a fallback date
      specsWithDates.push({ spec, date: new Date(0) });
    }
  }

  specsWithDates.sort((a, b) => b.date.getTime() - a.date.getTime());
  return specsWithDates[0]?.spec || null;
}

async function getLastCommitInfo(filePath: string): Promise<string> {
  try {
    const { stdout } = await execAsync(`git log -1 --format="%h %s" -- "${filePath}"`);
    return stdout.trim() || "unknown";
  } catch {
    return "unknown";
  }
}

function outputReviewPrompt(
  issue: Issue,
  spec: SpecInfo,
  specContent: string,
  lastCommit: string,
  hasExistingReview: boolean
): void {
  const date = getCurrentDate();

  console.log("=".repeat(80));
  console.log(`SPEC REVIEW: ${spec.name}`);
  console.log("=".repeat(80));
  console.log();
  console.log(`Issue: ${issue.name} (3-in-progress)`);
  console.log(`Spec: ${spec.name}.spec.md`);
  console.log(`Status: ${spec.frontmatter.status}`);
  console.log(`Last commit: ${lastCommit}`);
  console.log();
  console.log("-".repeat(80));
  console.log("SPEC CONTENT:");
  console.log("-".repeat(80));
  console.log();
  console.log(specContent);
  console.log();
  console.log("-".repeat(80));
  console.log("REVIEW INSTRUCTIONS:");
  console.log("-".repeat(80));
  console.log();
  console.log("1. IMPLEMENTATION VERIFICATION:");
  console.log("   - Read the Implementation Notes to locate the relevant source files");
  console.log("   - For each acceptance criterion, verify the code implements it");
  console.log("   - Reference specific files and line numbers as evidence");
  console.log();
  console.log("2. CODE QUALITY AUDIT:");
  console.log("   - Check adherence to codebase patterns (traits, error types, etc.)");
  console.log("   - Verify error handling and type safety");
  console.log("   - Check test coverage matches test cases in the spec");
  console.log("   - For Rust: check ownership, thread safety, coverage exclusions");
  console.log("   - For TypeScript: check types, hooks usage, coverage exclusions");
  console.log();
  console.log("3. INTEGRATION VERIFICATION:");
  console.log("   - Check if mocked components are instantiated in production code (lib.rs, main.tsx)");
  console.log('   - Search for comments like "handled separately", "managed elsewhere" - verify spec refs');
  console.log("   - Verify integration tests exist and pass (or mark N/A for unit-only specs)");
  console.log();
  console.log("4. GENERATE REVIEW SECTION:");
  console.log("   Use this template and append to the spec file:");
  console.log();
  console.log("```markdown");
  console.log("## Review");
  console.log();
  console.log(`**Reviewed:** ${date}`);
  console.log("**Reviewer:** Claude");
  console.log();
  console.log("### Acceptance Criteria Verification");
  console.log();
  console.log("| Criterion | Status | Evidence |");
  console.log("|-----------|--------|----------|");
  console.log("| [criterion text] | PASS/FAIL | file:line reference |");
  console.log();
  console.log("### Test Coverage Audit");
  console.log();
  console.log("| Test Case | Status | Location |");
  console.log("|-----------|--------|----------|");
  console.log("| [test case text] | PASS/MISSING | test file reference |");
  console.log();
  console.log("### Code Quality");
  console.log();
  console.log("**Strengths:**");
  console.log("- [bullet points]");
  console.log();
  console.log("**Concerns:**");
  console.log("- [bullet points if any, or 'None identified']");
  console.log();
  console.log("### Integration Verification");
  console.log();
  console.log("| Check | Status | Evidence |");
  console.log("|-------|--------|----------|");
  console.log('| Mocked components instantiated in production? | PASS/FAIL/N/A | file:line or "no mocks used" |');
  console.log('| Any "handled separately" without spec reference? | PASS/FAIL | List any untracked deferrals |');
  console.log('| Integration test exists and passes? | PASS/FAIL/N/A | test file:line or "unit-only spec" |');
  console.log();
  console.log("### Deferral Audit");
  console.log();
  console.log("| Deferral Statement | Location | Tracking Reference |");
  console.log("|--------------------|----------|-------------------|");
  console.log('| [quote or "None found"] | file:line | spec-name or MISSING |');
  console.log();
  console.log("### Verdict");
  console.log();
  console.log("[APPROVED/NEEDS_WORK] - [1-2 sentence summary]");
  console.log("```");
  console.log();
  console.log("=".repeat(80));
  console.log("ACTION REQUIRED:");

  if (hasExistingReview) {
    console.log(`Replace the existing ## Review section in:`);
  } else {
    console.log(`Append the Review section to:`);
  }
  console.log(`  ${spec.path}`);
  console.log("=".repeat(80));
}
