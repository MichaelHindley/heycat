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
    } catch (_error) {
      dir = dirname(dir);
    }
  }
  throw new Error("Could not find project root (no .git directory found)");
}

export async function fileExists(path: string): Promise<boolean> {
  try {
    await stat(path);
    return true;
  } catch (_error) {
    return false;
  }
}

// ============================================================================
// Rust Test Module Derivation
// ============================================================================

/**
 * Derive the Rust test module filter pattern from a source file path.
 * Used for filtering which tests to run with cargo-llvm-cov.
 *
 * Supports two test patterns:
 * 1. Inline tests: `#[cfg(test)] mod tests { }` within source files
 * 2. Separate test files: `foo_test.rs` alongside `foo.rs`
 *
 * Examples:
 * - src-tauri/src/lib.rs → "tests::" (inline tests at crate root)
 * - src-tauri/src/main.rs → "tests::" (inline tests at crate root)
 * - src-tauri/src/foo.rs → "foo::foo_test" (if foo_test.rs exists) or "foo::tests::"
 * - src-tauri/src/bar/mod.rs → "bar::tests::" (inline tests in bar module)
 * - src-tauri/src/bar/baz.rs → "bar::baz_test" (if baz_test.rs exists) or "bar::baz::tests::"
 * - src-tauri/src/bar/baz_test.rs → "bar::baz_test" (tests directly in test module)
 *
 * @param filePath - The source file path
 * @param projectRoot - The project root directory (optional, for checking test file existence)
 */
export async function deriveRustTestModule(
  filePath: string,
  projectRoot?: string
): Promise<string | null> {
  // Only process .rs files in src-tauri/src/
  if (!filePath.endsWith(".rs") || !filePath.includes("src-tauri/src/")) {
    return null;
  }

  // Extract relative path from src-tauri/src/
  const match = filePath.match(/src-tauri\/src\/(.+)\.rs$/);
  if (!match) return null;

  const relativePath = match[1];

  // lib.rs and main.rs → tests at crate root (inline only)
  if (relativePath === "lib" || relativePath === "main") {
    return "tests::";
  }

  // If this IS a test file (*_test.rs), return the test module path directly
  // Tests in *_test.rs are at module level, not in a nested tests:: submodule
  if (relativePath.endsWith("_test")) {
    const modulePath = relativePath.replace(/\//g, "::");
    return modulePath;
  }

  // mod.rs → parent directory is the module (inline tests only for now)
  if (relativePath.endsWith("/mod")) {
    const parent = relativePath.replace(/\/mod$/, "").replace(/\//g, "::");
    return `${parent}::tests::`;
  }

  // For regular source files (foo.rs), check if a corresponding test file exists
  if (projectRoot) {
    const dir = dirname(filePath);
    const baseName = basename(filePath, ".rs");
    const testFilePath = join(dir, `${baseName}_test.rs`);
    const absoluteTestPath = join(projectRoot, testFilePath);

    if (await fileExists(absoluteTestPath)) {
      // Test file exists - return test module path (without ::tests::)
      const testModulePath = relativePath.replace(/\//g, "::") + "_test";
      return testModulePath;
    }
  }

  // No test file found - assume inline tests
  const modulePath = relativePath.replace(/\//g, "::");
  return `${modulePath}::tests::`;
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
 * Find test files for multiple source files (parallelized for performance)
 */
export async function findTestFiles(
  sourceFiles: string[],
  projectRoot: string
): Promise<string[]> {
  // Run all lookups in parallel for better performance
  const results = await Promise.all(
    sourceFiles.map((file) => findTestFile(file, projectRoot))
  );

  // Filter out nulls and deduplicate
  const seen = new Set<string>();
  const testFiles: string[] = [];

  for (const testFile of results) {
    if (testFile && !seen.has(testFile)) {
      seen.add(testFile);
      testFiles.push(testFile);
    }
  }

  return testFiles;
}

// ============================================================================
// Git Operations
// ============================================================================

/**
 * Get list of changed files using git status -s for compact output.
 * Uses short format which is more context-efficient than git diff --name-only.
 *
 * Format: "XY filename" where X=staged status, Y=unstaged status
 * Examples: "M  file.ts" (modified staged), " M file.ts" (modified unstaged)
 *
 * For untracked directories (shown as "?? dir/"), expands to list all files inside.
 */
export async function getChangedFiles(staged: boolean = false): Promise<string[]> {
  const { $ } = await import("bun");

  try {
    const result = await $`git status -s`.quiet();

    if (result.exitCode !== 0) {
      // If git status fails (unlikely), fall back to ls-files for new repos
      const allFiles = await $`git ls-files`.quiet();
      return allFiles.text().trim().split("\n").filter(Boolean);
    }

    const output = result.text().trimEnd();
    if (!output) return [];

    const paths = output
      .split("\n")
      .filter(Boolean)
      .map((line) => {
        // Parse "XY filename" or "XY orig -> renamed"
        // First 2 chars are status, then space(s), then filename
        const match = line.match(/^..\s+(.+?)(?:\s+->\s+(.+))?$/);
        if (!match) return null;
        // For renames, return the new filename (match[2]), otherwise original (match[1])
        return match[2] || match[1];
      })
      .filter((f): f is string => f !== null);

    // Expand directories to list all files inside
    const expandedPaths: string[] = [];
    for (const path of paths) {
      if (path.endsWith("/")) {
        // Untracked directory - list all files recursively
        try {
          const filesResult = await $`find ${path} -type f`.quiet();
          const files = filesResult.text().trim().split("\n").filter(Boolean);
          expandedPaths.push(...files);
        } catch {
          // If find fails, keep the directory path
          expandedPaths.push(path);
        }
      } else {
        expandedPaths.push(path);
      }
    }

    return expandedPaths;
  } catch (error) {
    console.error(
      "TCR: Failed to get changed files:",
      error instanceof Error ? error.message : "Unknown error"
    );
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
    // Note: Bun's $ template literal auto-escapes interpolated values, preventing shell injection
    const commitMessage = `${WIP_PREFIX}${message}`;
    await $`git commit -m ${commitMessage}`.quiet();

    // Get the commit hash
    const hash = await $`git rev-parse --short HEAD`.quiet();
    return hash.text().trim();
  } catch (error) {
    console.error(
      "TCR: Failed to create WIP commit:",
      error instanceof Error ? error.message : "Unknown error"
    );
    return null;
  }
}

// ============================================================================
// Target Detection
// ============================================================================

/**
 * Determine which test target(s) to run based on changed files.
 * Returns "frontend" as the default when no specific target is detected.
 */
export function determineTarget(changedFiles: string[]): TestTarget {
  // Empty file list defaults to frontend
  if (changedFiles.length === 0) {
    return "frontend";
  }

  const hasFrontend = changedFiles.some((f) =>
    FRONTEND_EXTENSIONS.some((ext) => f.endsWith(ext) && f.startsWith("src/"))
  );

  const hasBackend = changedFiles.some((f) =>
    BACKEND_EXTENSIONS.some((ext) => f.endsWith(ext) && f.startsWith("src-tauri/"))
  );

  if (hasFrontend && hasBackend) return "both";
  if (hasBackend) return "backend";
  if (hasFrontend) return "frontend";

  // No recognized source files - default to frontend for safety
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

// ============================================================================
// Error Logging
// ============================================================================

const ERROR_LOG_FILE = ".tcr-errors.log";
const MAX_ERROR_LOG_ENTRIES = 10;

interface ErrorLogEntry {
  timestamp: string;
  error: string;
  context?: string;
}

/**
 * Log an error to the persistent error log file.
 * Keeps only the most recent entries to prevent unbounded growth.
 */
export async function logError(
  projectRoot: string,
  error: string,
  context?: string
): Promise<void> {
  const logPath = join(projectRoot, ERROR_LOG_FILE);

  try {
    // Read existing entries
    let entries: ErrorLogEntry[] = [];
    const file = Bun.file(logPath);
    if (await file.exists()) {
      try {
        entries = await file.json();
      } catch {
        // Invalid JSON, start fresh
        entries = [];
      }
    }

    // Add new entry
    entries.push({
      timestamp: getCurrentTimestamp(),
      error,
      context,
    });

    // Keep only recent entries
    if (entries.length > MAX_ERROR_LOG_ENTRIES) {
      entries = entries.slice(-MAX_ERROR_LOG_ENTRIES);
    }

    // Write back
    await Bun.write(logPath, JSON.stringify(entries, null, 2));
  } catch (writeError) {
    // Don't let error logging itself cause issues
    console.error(
      "TCR: Failed to write error log:",
      writeError instanceof Error ? writeError.message : "Unknown error"
    );
  }
}

/**
 * Read recent errors from the error log.
 * Returns empty array if log doesn't exist or can't be read.
 */
export async function readErrorLog(
  projectRoot: string
): Promise<ErrorLogEntry[]> {
  const logPath = join(projectRoot, ERROR_LOG_FILE);

  try {
    const file = Bun.file(logPath);
    if (!(await file.exists())) {
      return [];
    }
    return await file.json();
  } catch {
    return [];
  }
}

/**
 * Clear the error log.
 */
export async function clearErrorLog(projectRoot: string): Promise<void> {
  const logPath = join(projectRoot, ERROR_LOG_FILE);
  const { unlink } = await import("node:fs/promises");

  try {
    await unlink(logPath);
  } catch {
    // File doesn't exist, that's fine
  }
}
