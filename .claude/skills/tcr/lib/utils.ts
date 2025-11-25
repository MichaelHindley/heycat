import { stat } from "node:fs/promises";
import { join, dirname, basename, extname } from "node:path";
import {
  FRONTEND_EXTENSIONS,
  BACKEND_EXTENSIONS,
  WIP_PREFIX,
  type TestTarget,
} from "./types";

// ============================================================================
// File System Functions
// ============================================================================

export async function findProjectRoot(): Promise<string> {
  let dir = process.cwd();
  while (dir !== "/") {
    try {
      // Look for .git directory as project root indicator
      await stat(join(dir, ".git"));
      return dir;
    } catch {
      dir = dirname(dir);
    }
  }
  throw new Error("Could not find project root (no .git directory found)");
}

export async function fileExists(path: string): Promise<boolean> {
  try {
    await stat(path);
    return true;
  } catch {
    return false;
  }
}

// ============================================================================
// Test File Discovery (Convention-based)
// ============================================================================

/**
 * Find the test file for a given source file using convention:
 * foo.ts -> foo.test.ts or foo.spec.ts
 */
export async function findTestFile(
  sourceFile: string,
  projectRoot: string
): Promise<string | null> {
  const ext = extname(sourceFile);
  const base = basename(sourceFile, ext);
  const dir = dirname(sourceFile);

  // Try .test and .spec variants
  const testVariants = [`${base}.test${ext}`, `${base}.spec${ext}`];

  for (const variant of testVariants) {
    const testPath = join(projectRoot, dir, variant);
    if (await fileExists(testPath)) {
      return join(dir, variant);
    }
  }

  return null;
}

/**
 * Find test files for multiple source files
 */
export async function findTestFiles(
  sourceFiles: string[],
  projectRoot: string
): Promise<string[]> {
  const testFiles: string[] = [];

  for (const file of sourceFiles) {
    const testFile = await findTestFile(file, projectRoot);
    if (testFile && !testFiles.includes(testFile)) {
      testFiles.push(testFile);
    }
  }

  return testFiles;
}

// ============================================================================
// Git Operations
// ============================================================================

/**
 * Get list of changed files from git diff
 */
export async function getChangedFiles(staged: boolean = false): Promise<string[]> {
  const { $ } = await import("bun");

  try {
    const args = staged
      ? ["diff", "--cached", "--name-only"]
      : ["diff", "--name-only", "HEAD"];

    const result = await $`git ${args}`.quiet();

    if (result.exitCode !== 0) {
      // If HEAD doesn't exist (new repo), get all files
      if (!staged) {
        const allFiles = await $`git ls-files`.quiet();
        return allFiles.text().trim().split("\n").filter(Boolean);
      }
      return [];
    }

    return result.text().trim().split("\n").filter(Boolean);
  } catch {
    return [];
  }
}

/**
 * Create a WIP commit with the given message
 */
export async function createWipCommit(message: string): Promise<string | null> {
  const { $ } = await import("bun");

  try {
    // Stage all changes
    await $`git add -A`.quiet();

    // Check if there's anything to commit
    const status = await $`git status --porcelain`.quiet();
    if (!status.text().trim()) {
      return null; // Nothing to commit
    }

    // Create commit
    const commitMessage = `${WIP_PREFIX}${message}`;
    await $`git commit -m ${commitMessage}`.quiet();

    // Get the commit hash
    const hash = await $`git rev-parse --short HEAD`.quiet();
    return hash.text().trim();
  } catch {
    return null;
  }
}

// ============================================================================
// Target Detection
// ============================================================================

/**
 * Determine which test target(s) to run based on changed files
 */
export function determineTarget(changedFiles: string[]): TestTarget {
  const hasFrontend = changedFiles.some((f) =>
    FRONTEND_EXTENSIONS.some((ext) => f.endsWith(ext) && f.startsWith("src/"))
  );

  const hasBackend = changedFiles.some((f) =>
    BACKEND_EXTENSIONS.some((ext) => f.endsWith(ext) && f.startsWith("src-tauri/"))
  );

  if (hasFrontend && hasBackend) return "both";
  if (hasBackend) return "backend";
  return "frontend";
}

// ============================================================================
// Date Functions
// ============================================================================

export function getCurrentTimestamp(): string {
  return new Date().toISOString();
}

// ============================================================================
// Stdin Reading
// ============================================================================

export async function readStdin<T>(): Promise<T> {
  const chunks: Buffer[] = [];
  for await (const chunk of process.stdin) {
    chunks.push(chunk as Buffer);
  }
  return JSON.parse(Buffer.concat(chunks).toString()) as T;
}
