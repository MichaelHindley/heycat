import { parseArgs } from "node:util";
import { rename, mkdir } from "node:fs/promises";
import { join, basename } from "node:path";
import { ARCHIVE_DIR } from "../lib/types";
import { getCurrentDate, findProjectRoot, findIssue } from "../lib/utils";

export async function handleArchive(args: string[]): Promise<void> {
  const { positionals } = parseArgs({
    args,
    allowPositionals: true,
  });

  const [name] = positionals;

  if (!name) {
    console.error("Usage: agile.ts archive <name>");
    process.exit(1);
  }

  const projectRoot = await findProjectRoot();
  const issue = await findIssue(projectRoot, name);

  if (!issue) {
    console.error(`Issue not found: ${name}`);
    process.exit(1);
  }

  // Create archive directory
  const archiveDir = join(projectRoot, ARCHIVE_DIR);
  await mkdir(archiveDir, { recursive: true });

  // Archive with timestamp to allow re-archiving
  const slug = basename(issue.filename, ".md");
  const timestamp = getCurrentDate();
  const archiveFilename = `${slug}-${timestamp}.md`;
  const archivePath = join(archiveDir, archiveFilename);

  await rename(issue.path, archivePath);

  console.log(`Archived: ${slug}`);
  console.log(`  -> ${ARCHIVE_DIR}/${archiveFilename}`);
}
