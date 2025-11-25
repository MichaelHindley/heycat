import { parseArgs } from "node:util";
import { STAGE_NAMES, VALID_TRANSITIONS, type Stage } from "../lib/types";
import { findProjectRoot, findIssue, parseIssueMeta } from "../lib/utils";
import {
  analyzeIssueFile,
  STAGE_GUIDANCE,
  validateForTransition,
} from "../lib/analysis";

export async function handleWork(args: string[]): Promise<void> {
  const { positionals } = parseArgs({
    args,
    allowPositionals: true,
  });

  const [name] = positionals;

  if (!name) {
    console.error("Usage: agile.ts work <name>");
    console.error("Analyze an issue and get stage-appropriate guidance.");
    process.exit(1);
  }

  const projectRoot = await findProjectRoot();
  const issue = await findIssue(projectRoot, name);

  if (!issue) {
    console.error(`Issue not found: ${name}`);
    process.exit(1);
  }

  // Parse issue metadata
  const meta = await parseIssueMeta(issue.path);
  const analysis = await analyzeIssueFile(issue.path, meta.owner);
  const guidance = STAGE_GUIDANCE[issue.stage];

  // Determine next stage
  const allowedTransitions = VALID_TRANSITIONS[issue.stage];
  const forwardStage = allowedTransitions.find(
    (s) => parseInt(s[0]) > parseInt(issue.stage[0])
  );

  // Validate readiness for next stage
  let readiness = { valid: true, missing: [] as string[] };
  if (forwardStage) {
    readiness = validateForTransition(forwardStage, analysis);
  }

  // Output structured analysis
  console.log("=".repeat(80));
  console.log(`WORK SESSION: ${name}`);
  console.log("=".repeat(80));
  console.log();

  console.log("ISSUE DETAILS");
  console.log(`  Type:     ${meta.type}`);
  console.log(`  Stage:    ${issue.stage} (${STAGE_NAMES[issue.stage]})`);
  console.log(`  Owner:    ${meta.owner}`);
  console.log(`  Created:  ${meta.created}`);
  console.log(`  Path:     ${issue.path}`);
  console.log();

  console.log("ANALYSIS");
  if (analysis.incompleteSections.length > 0) {
    console.log("  Incomplete Sections:");
    for (const section of analysis.incompleteSections) {
      console.log(`    - ${section} (has placeholder text)`);
    }
  } else {
    console.log("  All sections complete");
  }
  console.log();

  console.log(`  Definition of Done: ${analysis.dod.completed}/${analysis.dod.total} completed`);
  for (const item of analysis.dod.items) {
    console.log(`    [${item.checked ? "x" : " "}] ${item.text}`);
  }
  console.log();

  console.log(`STAGE GUIDANCE (${STAGE_NAMES[issue.stage]})`);
  console.log(`  Focus: ${guidance.focus}`);
  console.log();
  console.log("  Suggested Actions:");
  for (let i = 0; i < guidance.actions.length; i++) {
    console.log(`    ${i + 1}. ${guidance.actions[i]}`);
  }
  console.log();

  if (forwardStage) {
    console.log("READINESS TO ADVANCE");
    console.log(`  Status: ${readiness.valid ? "READY" : "NOT READY"}`);
    if (readiness.missing.length > 0) {
      console.log("  Blockers:");
      for (const m of readiness.missing) {
        console.log(`    - ${m}`);
      }
    }
    console.log();
    console.log(`  Next Stage: ${forwardStage} (${STAGE_NAMES[forwardStage]})`);
    console.log(`  Command: bun .claude/skills/agile/agile.ts move ${name} ${forwardStage}`);
  } else {
    console.log("COMPLETION");
    console.log("  This issue is in the final stage (Done).");
    console.log("  Consider archiving:");
    console.log(`  Command: bun .claude/skills/agile/agile.ts archive ${name}`);
  }

  console.log();
  console.log("=".repeat(80));
}
