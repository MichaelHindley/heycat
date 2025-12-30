#!/usr/bin/env bun
/**
 * Delete the current worktree and clean up all associated data.
 *
 * Must be run from within a worktree, not the main repository.
 * This is part of the "cattle" worktree workflow - worktrees are ephemeral
 * and deleted after their PR is merged.
 *
 * Usage:
 *   bun scripts/close-worktree.ts           # Delete worktree and cleanup
 *   bun scripts/close-worktree.ts --force   # Skip confirmation
 *   bun scripts/close-worktree.ts --help    # Show help
 *
 * This script:
 * 1. Detects worktree context
 * 2. Removes git worktree
 * 3. Cleans up data directories
 * 4. Prints navigation instructions
 */

import { existsSync, rmSync } from "fs";
import { homedir } from "os";
import { resolve } from "path";
import { createInterface } from "readline";
import { detectWorktreeContext, type WorktreeInfo } from "./lib/worktree";
import { getDataDir, getConfigDir } from "./cleanup-worktree";

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

interface Flags {
  force: boolean;
  help: boolean;
}

/**
 * Parse command line arguments.
 */
export function parseArgs(args: string[]): Flags {
  const flags: Flags = {
    force: false,
    help: false,
  };

  for (const arg of args) {
    if (arg === "--force" || arg === "-f") {
      flags.force = true;
    } else if (arg === "--help" || arg === "-h") {
      flags.help = true;
    }
  }

  return flags;
}

/**
 * Print help message.
 */
function printHelp(): void {
  log(`
${colors.bold}Usage:${colors.reset} bun scripts/close-worktree.ts [options]

${colors.bold}Description:${colors.reset}
  Deletes the current worktree and cleans up all associated data.
  Must be run from within a worktree, not the main repository.

${colors.bold}Options:${colors.reset}
  --force, -f     Skip confirmation prompt
  --help, -h      Show this help message

${colors.bold}What gets deleted:${colors.reset}
  - Git worktree (the directory and git tracking)
  - Data directory: ~/.local/share/heycat-{identifier}/
  - Config directory: ~/.config/heycat-{identifier}/
  - Settings file: ~/Library/Application Support/com.heycat.app/settings-{identifier}.json

${colors.bold}Examples:${colors.reset}
  ${colors.cyan}bun scripts/close-worktree.ts${colors.reset}
    Delete worktree with confirmation

  ${colors.cyan}bun scripts/close-worktree.ts --force${colors.reset}
    Delete worktree without confirmation

${colors.bold}Note:${colors.reset}
  This is part of the "cattle" worktree workflow. Run this after your PR is merged.
  After deletion, navigate to the main repository with the printed cd command.
`);
}

/**
 * Prompt user for confirmation.
 */
async function confirm(message: string): Promise<boolean> {
  const rl = createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  return new Promise((res) => {
    rl.question(`${message} [y/N]: `, (answer) => {
      rl.close();
      res(answer.toLowerCase() === "y" || answer.toLowerCase() === "yes");
    });
  });
}

/**
 * Get the heycat application support directory path.
 */
function getAppSupportDir(): string {
  const home = homedir();
  if (process.platform === "darwin") {
    return resolve(home, "Library/Application Support/com.heycat.app");
  }
  return resolve(home, ".local/share/com.heycat.app");
}

/**
 * Remove git worktree.
 */
async function removeGitWorktree(
  mainRepoPath: string,
  worktreePath: string
): Promise<{ success: boolean; error?: string }> {
  const result = await Bun.spawn(["git", "-C", mainRepoPath, "worktree", "remove", worktreePath], {
    stdout: "pipe",
    stderr: "pipe",
  });

  const stderr = await new Response(result.stderr).text();
  const exitCode = await result.exited;

  return {
    success: exitCode === 0,
    error: exitCode !== 0 ? stderr.trim() : undefined,
  };
}

/**
 * Clean up worktree data directories and settings file.
 */
function cleanupWorktreeData(identifier: string): { deleted: string[]; failed: string[] } {
  const deleted: string[] = [];
  const failed: string[] = [];

  // Data directory: ~/.local/share/heycat-{identifier}/
  const dataDir = resolve(getDataDir(), `heycat-${identifier}`);
  if (existsSync(dataDir)) {
    try {
      rmSync(dataDir, { recursive: true, force: true });
      deleted.push(dataDir);
    } catch (e) {
      failed.push(dataDir);
    }
  }

  // Config directory: ~/.config/heycat-{identifier}/
  const configDir = resolve(getConfigDir(), `heycat-${identifier}`);
  if (existsSync(configDir)) {
    try {
      rmSync(configDir, { recursive: true, force: true });
      deleted.push(configDir);
    } catch (e) {
      failed.push(configDir);
    }
  }

  // Settings file: ~/Library/Application Support/com.heycat.app/settings-{identifier}.json
  const settingsPath = resolve(getAppSupportDir(), `settings-${identifier}.json`);
  if (existsSync(settingsPath)) {
    try {
      rmSync(settingsPath);
      deleted.push(settingsPath);
    } catch (e) {
      failed.push(settingsPath);
    }
  }

  return { deleted, failed };
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
    log("\nTo close a worktree, navigate to the worktree directory first.");
    process.exit(1);
  }

  const { identifier, mainRepoPath, worktreePath } = worktree;

  log(`\n${colors.bold}Closing worktree${colors.reset}\n`);
  info(`Identifier: ${identifier}`);
  info(`Worktree path: ${worktreePath}`);
  info(`Main repo: ${mainRepoPath}`);

  // Confirm deletion unless --force
  if (!flags.force) {
    log("");
    const confirmed = await confirm(`Delete worktree '${identifier}' and all associated data?`);
    if (!confirmed) {
      info("\nCancelled.");
      process.exit(0);
    }
  }

  // Remove git worktree
  log(`\n${colors.bold}Removing git worktree...${colors.reset}`);
  const removeResult = await removeGitWorktree(mainRepoPath, worktreePath);

  if (!removeResult.success) {
    error(`Failed to remove git worktree: ${removeResult.error}`);
    warn("\nYou may need to manually remove it:");
    log(`  git -C ${mainRepoPath} worktree remove ${worktreePath}`);
    log("\nIf the worktree has uncommitted changes, use --force:");
    log(`  git -C ${mainRepoPath} worktree remove --force ${worktreePath}`);
    process.exit(1);
  }
  success("  Git worktree removed");

  // Clean up data directories
  log(`\n${colors.bold}Cleaning up data...${colors.reset}`);
  const { deleted, failed } = cleanupWorktreeData(identifier);

  for (const path of deleted) {
    success(`  Deleted: ${path}`);
  }

  if (failed.length > 0) {
    for (const path of failed) {
      warn(`  Failed to delete: ${path}`);
    }
    warn("\nSome cleanup failed. Run to clean up orphaned data:");
    log(`  bun scripts/cleanup-worktree.ts --orphaned`);
  }

  // Print navigation instructions
  log(`
${colors.green}${colors.bold}Worktree closed successfully!${colors.reset}

${colors.bold}Navigate to main repository:${colors.reset}
  ${colors.cyan}cd ${mainRepoPath}${colors.reset}
`);
}

// Only run main when executed directly, not when imported as a module
if (import.meta.main) {
  main().catch((err) => {
    error(err.message || String(err));
    process.exit(1);
  });
}
