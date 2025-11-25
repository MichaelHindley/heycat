import { parseArgs } from "node:util";
import { rename, mkdir } from "node:fs/promises";
import { join, basename } from "node:path";
import {
  STAGES,
  STAGE_NAMES,
  VALID_TRANSITIONS,
  AGILE_DIR,
  type Stage,
} from "../lib/types";
import { isValidStage, findProjectRoot, findIssue } from "../lib/utils";

export async function handleMove(args: string[]): Promise<void> {
  const { positionals } = parseArgs({
    args,
    allowPositionals: true,
  });

  const [name, toStage] = positionals;

  if (!name || !toStage) {
    console.error("Usage: agile.ts move <name> <stage>");
    console.error(`Stages: ${STAGES.join(", ")}`);
    process.exit(1);
  }

  if (!isValidStage(toStage)) {
    console.error(`Invalid stage: "${toStage}". Valid stages: ${STAGES.join(", ")}`);
    process.exit(1);
  }

  const projectRoot = await findProjectRoot();
  const issue = await findIssue(projectRoot, name);

  if (!issue) {
    console.error(`Issue not found: ${name}`);
    process.exit(1);
  }

  if (issue.stage === toStage) {
    console.log(`Issue is already in ${STAGE_NAMES[toStage]}`);
    return;
  }

  // Validate transition
  const allowedTransitions = VALID_TRANSITIONS[issue.stage];
  if (!allowedTransitions.includes(toStage)) {
    console.error(
      `Invalid transition: ${STAGE_NAMES[issue.stage]} -> ${STAGE_NAMES[toStage]}`
    );
    console.error(
      `Allowed from ${STAGE_NAMES[issue.stage]}: ${allowedTransitions.map((s) => STAGE_NAMES[s]).join(", ")}`
    );
    process.exit(1);
  }

  // Move the file
  const targetDir = join(projectRoot, AGILE_DIR, toStage);
  await mkdir(targetDir, { recursive: true });
  const targetPath = join(targetDir, issue.filename);
  await rename(issue.path, targetPath);

  console.log(`Moved: ${basename(issue.filename, ".md")}`);
  console.log(`  ${STAGE_NAMES[issue.stage]} -> ${STAGE_NAMES[toStage]}`);
}
