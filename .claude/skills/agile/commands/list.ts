import { parseArgs } from "node:util";
import { readdir } from "node:fs/promises";
import { join, basename } from "node:path";
import {
  STAGES,
  STAGE_NAMES,
  AGILE_DIR,
  type Stage,
} from "../lib/types";
import { isValidStage, findProjectRoot, parseIssueMeta } from "../lib/utils";

interface IssueInfo {
  name: string;
  stage: Stage;
  type: string;
  title: string;
  created: string;
  owner: string;
  path: string;
}

export async function handleList(args: string[]): Promise<void> {
  const { values } = parseArgs({
    args,
    options: {
      stage: { type: "string", short: "s" },
      format: { type: "string", short: "f", default: "table" },
    },
    allowPositionals: true,
  });

  const filterStage = values.stage;
  if (filterStage && !isValidStage(filterStage)) {
    console.error(`Invalid stage: "${filterStage}". Valid stages: ${STAGES.join(", ")}`);
    process.exit(1);
  }

  const format = values.format as string;
  if (format !== "table" && format !== "json") {
    console.error('Invalid format. Use "table" or "json"');
    process.exit(1);
  }

  const projectRoot = await findProjectRoot();
  const stagesToList = filterStage ? [filterStage as Stage] : STAGES;

  const allIssues: IssueInfo[] = [];

  for (const stage of stagesToList) {
    const stagePath = join(projectRoot, AGILE_DIR, stage);
    let files: string[] = [];
    try {
      files = await readdir(stagePath);
    } catch {
      // Stage directory might not exist
    }

    for (const file of files) {
      if (file.endsWith(".md") && file !== ".gitkeep") {
        const filePath = join(stagePath, file);
        const meta = await parseIssueMeta(filePath);
        allIssues.push({
          name: basename(file, ".md"),
          stage,
          type: meta.type,
          title: meta.title || basename(file, ".md"),
          created: meta.created,
          owner: meta.owner,
          path: `${AGILE_DIR}/${stage}/${file}`,
        });
      }
    }
  }

  if (format === "json") {
    console.log(JSON.stringify(allIssues, null, 2));
    return;
  }

  // Table format
  for (const stage of stagesToList) {
    const stageIssues = allIssues.filter((i) => i.stage === stage);
    console.log(`\n${STAGE_NAMES[stage]} (${stageIssues.length})`);
    console.log("─".repeat(50));

    if (stageIssues.length === 0) {
      console.log("  (empty)");
    } else {
      for (const issue of stageIssues) {
        const typeTag = `[${issue.type}]`.padEnd(10);
        const ownerTag = issue.owner && issue.owner !== "[Name]" ? ` (${issue.owner})` : "";
        console.log(`  ${typeTag} ${issue.name} - ${issue.title}${ownerTag}`);
      }
    }
  }
  console.log();
}
