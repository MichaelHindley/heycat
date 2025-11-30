import { readFile } from "node:fs/promises";
import { findProjectRoot } from "../lib/utils";
import { createIssueResolver } from "../lib/issue-resolver";
import type { Issue } from "../lib/types";

export async function handleDiscover(args: string[]): Promise<void> {
  const [issueName] = args;

  if (!issueName) {
    console.error("Usage: agile.ts discover <issue-name>");
    console.error("\nGuides BDD scenario creation through product research questions.");
    process.exit(1);
  }

  const projectRoot = await findProjectRoot();
  const resolver = createIssueResolver();

  const issue = await resolver.findIssue(projectRoot, issueName);

  if (!issue) {
    console.error(`Issue not found: "${issueName}"`);
    console.error("Run 'agile.ts list' to see available issues.");
    process.exit(1);
  }

  // Only features need BDD discovery
  if (issue.type !== "feature") {
    console.error(`BDD discovery is only for features, not ${issue.type}s.`);
    console.error("Bugs and tasks can be moved to todo without BDD scenarios.");
    process.exit(1);
  }

  // Read current feature content
  const content = await readFile(issue.mainFilePath, "utf-8");

  // Check if scenarios already exist
  const hasScenarios = detectExistingScenarios(content);

  // Output the discovery prompt
  outputDiscoveryPrompt(issue, content, hasScenarios);
}

function detectExistingScenarios(content: string): boolean {
  // Check for Given/When/Then patterns
  const hasGiven = /\bGiven\b/i.test(content);
  const hasWhen = /\bWhen\b/i.test(content);
  const hasThen = /\bThen\b/i.test(content);
  const hasBDDSection = /## BDD Scenarios/i.test(content);

  return hasBDDSection && hasGiven && hasWhen && hasThen;
}

function outputDiscoveryPrompt(issue: Issue, content: string, hasExistingScenarios: boolean): void {
  console.log("=".repeat(80));
  console.log(`BDD DISCOVERY SESSION: ${issue.name}`);
  console.log("=".repeat(80));
  console.log();
  console.log(`Feature: ${issue.mainFilePath}`);
  console.log(`Title: ${issue.meta.title}`);
  console.log(`Stage: ${issue.stage}`);
  console.log();

  if (hasExistingScenarios) {
    console.log("NOTE: This feature already has BDD scenarios. Discovery will help refine them.");
    console.log();
  }

  console.log("-".repeat(80));
  console.log("CURRENT FEATURE CONTENT:");
  console.log("-".repeat(80));
  console.log();
  console.log(content);
  console.log();
  console.log("-".repeat(80));
  console.log("DISCOVERY INSTRUCTIONS:");
  console.log("-".repeat(80));
  console.log();
  console.log("You are conducting product research to define BDD scenarios for this feature.");
  console.log("Ask the following questions ONE AT A TIME, waiting for the user's response");
  console.log("before proceeding to the next question. Probe deeper when answers are vague.");
  console.log();
  console.log("=".repeat(80));
  console.log("PHASE 1: USER PERSONAS");
  console.log("=".repeat(80));
  console.log();
  console.log("1. WHO is the primary user of this feature?");
  console.log("   (Ask about their role, technical level, goals, and context)");
  console.log();
  console.log("2. WHAT problem are they trying to solve?");
  console.log("   (Ask about the pain point, current workarounds, and desired outcome)");
  console.log();
  console.log("3. WHY is this problem important to solve now?");
  console.log("   (Ask about urgency, business impact, and blockers)");
  console.log();
  console.log("=".repeat(80));
  console.log("PHASE 2: HAPPY + FAILURE PATHS");
  console.log("=".repeat(80));
  console.log();
  console.log("4. Walk through the IDEAL successful experience:");
  console.log("   - What state exists before the user starts? (Given)");
  console.log("   - What action triggers the feature? (When)");
  console.log("   - What should happen as a result? (Then)");
  console.log();
  console.log("5. What VARIATIONS of the happy path exist?");
  console.log("   (Ask about different starting states, user types, or alternative flows)");
  console.log();
  console.log("6. What could go WRONG during this feature?");
  console.log("   (Ask about errors, edge cases, invalid inputs, system failures)");
  console.log();
  console.log("7. How should FAILURES be handled?");
  console.log("   (Ask about error messages, fallbacks, recovery paths)");
  console.log();
  console.log("=".repeat(80));
  console.log("PHASE 3: SCOPE BOUNDARIES");
  console.log("=".repeat(80));
  console.log();
  console.log("8. What is explicitly OUT OF SCOPE?");
  console.log("   (Ask about features to defer, adjacent functionality, future work)");
  console.log();
  console.log("9. What ASSUMPTIONS are we making?");
  console.log("   (Ask about preconditions, dependencies, system state requirements)");
  console.log();
  console.log("-".repeat(80));
  console.log("SYNTHESIS INSTRUCTIONS:");
  console.log("-".repeat(80));
  console.log();
  console.log("After gathering all responses, synthesize them into BDD scenarios.");
  console.log("Write the scenarios using this Gherkin format:");
  console.log();
  console.log("```gherkin");
  console.log("Feature: <Feature Title>");
  console.log();
  console.log("  Background:");
  console.log("    Given <shared preconditions that apply to all scenarios>");
  console.log();
  console.log("  Scenario: <descriptive name - the happy path>");
  console.log("    Given <initial state/context>");
  console.log("    When <user action or trigger>");
  console.log("    Then <expected outcome>");
  console.log("    And <additional outcome if needed>");
  console.log();
  console.log("  Scenario: <descriptive name - an edge case or failure>");
  console.log("    Given <initial state/context>");
  console.log("    When <action that triggers edge case>");
  console.log("    Then <expected error handling or fallback>");
  console.log("```");
  console.log();
  console.log("SCENARIO WRITING RULES:");
  console.log("- Each scenario must be independent and testable");
  console.log("- Use concrete examples, not abstractions");
  console.log('  (e.g., "Given the user has 3 items in cart" not "Given items exist")');
  console.log("- Include scenarios for each identified failure mode");
  console.log("- Name scenarios by the behavior they describe, not implementation");
  console.log("- Use Background for preconditions shared across ALL scenarios");
  console.log();
  console.log("=".repeat(80));
  console.log("ACTION REQUIRED:");
  console.log("=".repeat(80));
  console.log();
  console.log("1. Ask discovery questions one at a time (wait for responses)");
  console.log("2. Probe deeper when answers are vague or incomplete");
  console.log("3. Synthesize responses into BDD scenarios");

  if (hasExistingScenarios) {
    console.log("4. Use the Edit tool to REPLACE the ## BDD Scenarios section in:");
  } else {
    console.log("4. Use the Edit tool to ADD a ## BDD Scenarios section after ## Description in:");
  }
  console.log(`   ${issue.mainFilePath}`);
  console.log();
  console.log("The BDD Scenarios section should contain:");
  console.log("- A brief summary of the user persona and problem");
  console.log("- All scenarios in Gherkin format (Given/When/Then)");
  console.log("- Notes on scope boundaries and assumptions");
  console.log();
  console.log("=".repeat(80));
}
