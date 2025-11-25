import { parseArgs } from "node:util";
import { readFile, writeFile, mkdir } from "node:fs/promises";
import { join } from "node:path";
import {
  STAGES,
  TEMPLATES,
  AGILE_DIR,
  TEMPLATES_DIR,
  type Stage,
  type Template,
} from "../lib/types";
import {
  isValidStage,
  isValidTemplate,
  toKebabCase,
  toTitleCase,
  getCurrentDate,
  validateSlug,
  findProjectRoot,
  findIssue,
} from "../lib/utils";

export async function handleCreate(args: string[]): Promise<void> {
  const { values, positionals } = parseArgs({
    args,
    options: {
      title: { type: "string", short: "t" },
      owner: { type: "string", short: "o" },
      stage: { type: "string", short: "s", default: "1-backlog" },
    },
    allowPositionals: true,
  });

  const [type, name] = positionals;

  if (!type || !name) {
    console.error("Usage: agile.ts create <type> <name> [--title \"Title\"] [--owner \"Name\"]");
    console.error("Types: feature, bug, task");
    process.exit(1);
  }

  if (!isValidTemplate(type)) {
    console.error(`Invalid type: "${type}". Valid types: ${TEMPLATES.join(", ")}`);
    process.exit(1);
  }

  const stage = values.stage as string;
  if (!isValidStage(stage)) {
    console.error(`Invalid stage: "${stage}". Valid stages: ${STAGES.join(", ")}`);
    process.exit(1);
  }

  const slug = toKebabCase(name);
  validateSlug(slug);

  const projectRoot = await findProjectRoot();

  // Check if issue already exists
  const existing = await findIssue(projectRoot, slug);
  if (existing) {
    console.error(`Issue already exists: ${existing.stage}/${existing.filename}`);
    process.exit(1);
  }

  // Read template
  const templatePath = join(projectRoot, TEMPLATES_DIR, `${type}.md`);
  let content: string;
  try {
    content = await readFile(templatePath, "utf-8");
  } catch {
    console.error(`Template not found: ${templatePath}`);
    process.exit(1);
  }

  // Replace placeholders
  const title = values.title || toTitleCase(slug);
  const owner = values.owner || "[Name]";
  content = content.replace("[Title]", title);
  content = content.replace("YYYY-MM-DD", getCurrentDate());
  content = content.replace("[Name]", owner);

  // Write issue
  const targetDir = join(projectRoot, AGILE_DIR, stage);
  await mkdir(targetDir, { recursive: true });
  const targetPath = join(targetDir, `${slug}.md`);
  await writeFile(targetPath, content);

  console.log(`Created: ${AGILE_DIR}/${stage}/${slug}.md (${type})`);
}
