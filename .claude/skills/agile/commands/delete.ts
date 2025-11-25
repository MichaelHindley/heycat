import { parseArgs } from "node:util";
import { unlink } from "node:fs/promises";
import { basename } from "node:path";
import { STAGE_NAMES } from "../lib/types";
import { findProjectRoot, findIssue } from "../lib/utils";

export async function handleDelete(args: string[]): Promise<void> {
  const { positionals } = parseArgs({
    args,
    allowPositionals: true,
  });

  const [name] = positionals;

  if (!name) {
    console.error("Usage: agile.ts delete <name>");
    process.exit(1);
  }

  const projectRoot = await findProjectRoot();
  const issue = await findIssue(projectRoot, name);

  if (!issue) {
    console.error(`Issue not found: ${name}`);
    process.exit(1);
  }

  await unlink(issue.path);

  const slug = basename(issue.filename, ".md");
  console.log(`Deleted: ${slug} (was in ${STAGE_NAMES[issue.stage]})`);
}
