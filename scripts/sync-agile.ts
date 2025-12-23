#!/usr/bin/env bun
/**
 * Sync agile folder from a git worktree to the main branch.
 *
 * When working in a worktree, issue stage changes (moving files between folders)
 * commit to your feature branch. This script syncs those changes directly to main
 * without leaving the worktree.
 *
 * Usage:
 *   bun scripts/sync-agile.ts                    # Sync and commit to main
 *   bun scripts/sync-agile.ts --dry-run          # Preview changes
 *   bun scripts/sync-agile.ts -m "Move issue"    # Custom commit message
 *   bun scripts/sync-agile.ts --no-commit        # Stage only, don't commit
 *   bun scripts/sync-agile.ts --force            # Sync even if main is dirty
 */

import { existsSync, readFileSync, statSync } from "fs";
import { dirname, resolve } from "path";

// ANSI color codes for terminal output
const colors = {
  reset: "\x1b[0m",
  bold: "\x1b[1m",
  red: "\x1b[31m",
  green: "\x1b[32m",
  yellow: "\x1b[33m",
  cyan: "\x1b[36m",
  dim: "\x1b[2m",
};

function log(message: string): void {
  console.log(message);
}

function success(message: string): void {
  console.log(`${colors.green}${colors.bold}${message}${colors.reset}`);
}

function error(message: string): void {
  console.error(`${colors.red}${colors.bold}Error: ${message}${colors.reset}`);
}

function info(message: string): void {
  console.log(`${colors.cyan}${message}${colors.reset}`);
}

function warn(message: string): void {
  console.log(`${colors.yellow}${message}${colors.reset}`);
}

function dim(message: string): void {
  console.log(`${colors.dim}${message}${colors.reset}`);
}

/**
 * Information about the current worktree context.
 */
export interface WorktreeInfo {
  /** Worktree directory name (e.g., "heycat-feature-audio") */
  identifier: string;
  /** Path to the main repository */
  mainRepoPath: string;
  /** Current worktree path */
  worktreePath: string;
  /** Path to .git/worktrees/<name> */
  gitdirPath: string;
}

/**
 * Detect if we're running from a worktree and return context info.
 * Returns null if running from main repository.
 */
export async function detectWorktreeContext(): Promise<WorktreeInfo | null> {
  const gitPath = resolve(process.cwd(), ".git");

  if (!existsSync(gitPath)) {
    return null;
  }

  // Check if .git is a file (worktree) or directory (main repo)
  const stat = statSync(gitPath);
  if (stat.isDirectory()) {
    // Main repo - .git is a directory
    return null;
  }

  // Worktree - .git is a file containing gitdir reference
  const content = readFileSync(gitPath, "utf-8").trim();
  if (!content.startsWith("gitdir: ")) {
    return null;
  }

  // Extract gitdir path: "gitdir: /path/to/repo/.git/worktrees/<name>"
  const gitdirPath = content.substring("gitdir: ".length);

  // Navigate up from gitdir to find main repo:
  // .git/worktrees/<name> -> .git -> repo root
  const gitDir = dirname(dirname(gitdirPath)); // .git
  const mainRepoPath = dirname(gitDir); // repo root

  // The identifier is the last component of gitdirPath (worktree name)
  const identifier = gitdirPath.split("/").pop() || "";

  return {
    identifier,
    mainRepoPath,
    worktreePath: process.cwd(),
    gitdirPath,
  };
}

/**
 * Get the current branch of a repository.
 */
export async function getRepoBranch(repoPath: string): Promise<string> {
  const result = await Bun.spawn(["git", "-C", repoPath, "rev-parse", "--abbrev-ref", "HEAD"], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const output = await new Response(result.stdout).text();
  return output.trim();
}

/**
 * Check if the main repo has uncommitted changes in the agile/ folder.
 */
export async function checkMainRepoClean(mainRepoPath: string): Promise<boolean> {
  const result = await Bun.spawn(["git", "-C", mainRepoPath, "status", "--porcelain", "agile/"], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const output = await new Response(result.stdout).text();
  return output.trim() === "";
}

/**
 * Get a diff of what will change in the agile folder.
 */
export async function getAgileDiff(worktreePath: string, mainRepoPath: string): Promise<string> {
  // Use diff to compare the two directories
  const result = await Bun.spawn(["diff", "-rq", `${mainRepoPath}/agile`, `${worktreePath}/agile`], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const output = await new Response(result.stdout).text();
  return output.trim();
}

/**
 * Sync the agile folder from worktree to main repo using rsync.
 */
export async function syncAgileFolder(
  worktreePath: string,
  mainRepoPath: string,
  dryRun: boolean
): Promise<{ success: boolean; output: string }> {
  const args = ["-av", "--delete"];
  if (dryRun) {
    args.push("--dry-run");
  }
  args.push(`${worktreePath}/agile/`, `${mainRepoPath}/agile/`);

  const result = await Bun.spawn(["rsync", ...args], {
    stdout: "pipe",
    stderr: "pipe",
  });

  const stdout = await new Response(result.stdout).text();
  const stderr = await new Response(result.stderr).text();
  const exitCode = await result.exited;

  return {
    success: exitCode === 0,
    output: stdout + (stderr ? `\n${stderr}` : ""),
  };
}

/**
 * Check if there are staged changes in the agile folder.
 */
export async function hasStagedChanges(mainRepoPath: string): Promise<boolean> {
  const result = await Bun.spawn(["git", "-C", mainRepoPath, "diff", "--cached", "--name-only", "agile/"], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const output = await new Response(result.stdout).text();
  return output.trim() !== "";
}

/**
 * Stage and commit changes in the main repo.
 */
export async function commitChanges(
  mainRepoPath: string,
  message: string,
  worktreeIdentifier: string
): Promise<{ success: boolean; output: string }> {
  // Stage all agile changes
  const addResult = await Bun.spawn(["git", "-C", mainRepoPath, "add", "agile/"], {
    stdout: "pipe",
    stderr: "pipe",
  });
  await addResult.exited;

  // Check if there are changes to commit
  if (!(await hasStagedChanges(mainRepoPath))) {
    return { success: true, output: "No changes to commit" };
  }

  // Commit with the provided message
  const fullMessage = `${message}\n\nSynced from worktree: ${worktreeIdentifier}`;
  const commitResult = await Bun.spawn(["git", "-C", mainRepoPath, "commit", "-m", fullMessage], {
    stdout: "pipe",
    stderr: "pipe",
  });

  const stdout = await new Response(commitResult.stdout).text();
  const stderr = await new Response(commitResult.stderr).text();
  const exitCode = await commitResult.exited;

  return {
    success: exitCode === 0,
    output: stdout + (stderr ? `\n${stderr}` : ""),
  };
}

/**
 * Print help message.
 */
function printHelp(): void {
  log(`
${colors.bold}Usage:${colors.reset} bun scripts/sync-agile.ts [options]

${colors.bold}Description:${colors.reset}
  Syncs the agile/ folder from a worktree to the main branch.
  This keeps the kanban board state consistent across all worktrees.

${colors.bold}Options:${colors.reset}
  --dry-run        Preview changes without modifying anything
  --force          Sync even if main repo has uncommitted agile/ changes
  -m, --message    Custom commit message (default: "Sync agile/ from worktree")
  --no-commit      Stage changes but don't commit
  --help, -h       Show this help message

${colors.bold}Examples:${colors.reset}
  ${colors.cyan}bun scripts/sync-agile.ts${colors.reset}
    Sync agile/ and commit to main

  ${colors.cyan}bun scripts/sync-agile.ts --dry-run${colors.reset}
    Preview what would be synced

  ${colors.cyan}bun scripts/sync-agile.ts -m "Move feature-xyz to done"${colors.reset}
    Sync with custom commit message

${colors.bold}Note:${colors.reset}
  This script must be run from a worktree, not the main repository.
  Changes are committed to main locally - push manually when ready.
`);
}

interface Flags {
  dryRun: boolean;
  force: boolean;
  noCommit: boolean;
  message: string;
  help: boolean;
}

/**
 * Parse command line arguments.
 */
export function parseArgs(args: string[]): Flags {
  const flags: Flags = {
    dryRun: false,
    force: false,
    noCommit: false,
    message: "Sync agile/ from worktree",
    help: false,
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === "--dry-run") {
      flags.dryRun = true;
    } else if (arg === "--force" || arg === "-f") {
      flags.force = true;
    } else if (arg === "--no-commit") {
      flags.noCommit = true;
    } else if (arg === "-m" || arg === "--message") {
      flags.message = args[++i] || flags.message;
    } else if (arg === "--help" || arg === "-h") {
      flags.help = true;
    }
  }

  return flags;
}

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const flags = parseArgs(args);

  if (flags.help) {
    printHelp();
    process.exit(0);
  }

  // Detect worktree context
  const worktree = await detectWorktreeContext();
  if (!worktree) {
    error("This script must be run from a worktree, not the main repository.");
    log("\nIf you're in the main repo, use git directly to manage agile/ changes.");
    process.exit(1);
  }

  log(`\n${colors.bold}Syncing agile folder to main${colors.reset}\n`);
  info(`Worktree: ${worktree.identifier}`);
  info(`Main repo: ${worktree.mainRepoPath}`);

  // Check main repo branch
  const mainBranch = await getRepoBranch(worktree.mainRepoPath);
  if (mainBranch !== "main" && mainBranch !== "master") {
    warn(`\nWarning: Main repo is on branch '${mainBranch}', not main/master.`);
  }
  dim(`Target branch: ${mainBranch}`);

  // Check if main repo agile/ is clean
  if (!flags.force) {
    const isClean = await checkMainRepoClean(worktree.mainRepoPath);
    if (!isClean) {
      error("\nMain repo has uncommitted changes in agile/.");
      log("Use --force to sync anyway (overwrites main's changes).");
      process.exit(1);
    }
  }

  // Show what will change
  log("\n" + colors.bold + "Changes:" + colors.reset);
  const diff = await getAgileDiff(worktree.worktreePath, worktree.mainRepoPath);
  if (!diff) {
    success("\nAlready in sync - no changes needed.");
    process.exit(0);
  }

  // Parse and display diff nicely
  const diffLines = diff.split("\n").filter((line) => line.trim());
  for (const line of diffLines) {
    if (line.includes("Only in") && line.includes(worktree.worktreePath)) {
      // File only in worktree = will be added
      const file = line.split(": ").pop() || "";
      log(`  ${colors.green}+ ${file}${colors.reset}`);
    } else if (line.includes("Only in") && line.includes(worktree.mainRepoPath)) {
      // File only in main = will be deleted
      const file = line.split(": ").pop() || "";
      log(`  ${colors.red}- ${file}${colors.reset}`);
    } else if (line.includes(" differ")) {
      // Files differ = modified
      const match = line.match(/Files .+\/agile\/(.+) and .+ differ/);
      const file = match ? match[1] : line;
      log(`  ${colors.yellow}~ ${file}${colors.reset}`);
    }
  }

  if (flags.dryRun) {
    log("\n" + colors.dim + "(dry run - no changes made)" + colors.reset);
    process.exit(0);
  }

  // Sync files
  log("\n" + colors.bold + "Syncing..." + colors.reset);
  const syncResult = await syncAgileFolder(worktree.worktreePath, worktree.mainRepoPath, false);
  if (!syncResult.success) {
    error("Failed to sync files:");
    log(syncResult.output);
    process.exit(1);
  }

  if (flags.noCommit) {
    // Stage but don't commit
    await Bun.spawn(["git", "-C", worktree.mainRepoPath, "add", "agile/"], {
      stdout: "inherit",
      stderr: "inherit",
    });
    success("\nChanges staged in main repo (not committed).");
    log(`To commit: git -C ${worktree.mainRepoPath} commit -m "your message"`);
    process.exit(0);
  }

  // Commit changes
  log(colors.bold + "Committing..." + colors.reset);
  const commitResult = await commitChanges(worktree.mainRepoPath, flags.message, worktree.identifier);

  if (commitResult.output === "No changes to commit") {
    success("\nAlready in sync - no changes needed.");
    process.exit(0);
  }

  if (!commitResult.success) {
    error("Failed to commit:");
    log(commitResult.output);
    process.exit(1);
  }

  success("\nAgile folder synced to main!");
  dim(`Commit message: ${flags.message}`);
  log(`\nTo push: git -C ${worktree.mainRepoPath} push`);
}

// Only run main when executed directly, not when imported as a module
if (import.meta.main) {
  main().catch((err) => {
    error(err.message || String(err));
    process.exit(1);
  });
}
