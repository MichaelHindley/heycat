import { findProjectRoot, getChangedFiles, findTestFiles, determineTarget } from "../lib/utils";
import { runCoverageChecks } from "../lib/coverage";
import type { TestTarget } from "../lib/types";

/**
 * Handle the `tcr coverage` command.
 *
 * Usage:
 *   tcr coverage [--debug]
 *
 * Runs coverage checks on changed files only (same behavior as `tcr check`).
 * Target (frontend/backend/both) is auto-detected from changed files.
 *
 * Options:
 *   --debug, -d  Show detailed per-file coverage output
 */
export async function handleCoverage(args: string[]): Promise<void> {
  const projectRoot = await findProjectRoot();

  // Parse flags
  const debugMode = args.includes("--debug") || args.includes("-d");

  // Get changed files from git diff (same as check command)
  const changedFiles = await getChangedFiles(false);

  if (changedFiles.length === 0) {
    console.log("TCR Coverage: No changed files detected, nothing to check");
    process.exit(0);
  }

  console.log(`TCR Coverage: Found ${changedFiles.length} changed file(s)`);

  // Determine test target (frontend, backend, or both) from changed files
  const target = determineTarget(changedFiles);

  // Find test files for changed source files
  const testFiles = await findTestFiles(changedFiles, projectRoot);

  // Debug mode: show raw per-file coverage output
  if (debugMode) {
    console.log(`\nRunning ${target} coverage in debug mode (per-file details)...\n`);
    await runDebugCoverage(target, projectRoot, testFiles, changedFiles);
    return;
  }

  console.log(`\nRunning ${target} coverage checks...\n`);

  const result = await runCoverageChecks(target, testFiles, projectRoot, changedFiles);

  // Print the coverage report
  console.log(result.summary);

  // Exit with appropriate code
  process.exit(result.passed ? 0 : 1);
}

/**
 * Run coverage in debug mode with detailed per-file output.
 */
async function runDebugCoverage(
  target: TestTarget,
  projectRoot: string,
  testFiles: string[],
  changedFiles: string[]
): Promise<void> {
  const { $ } = await import("bun");
  let exitCode = 0;

  if (target === "frontend" || target === "both") {
    console.log("=".repeat(60));
    console.log("              FRONTEND COVERAGE (Debug)");
    console.log("=".repeat(60));
    console.log("");

    try {
      // Run vitest with text reporter for detailed output on specific test files
      const frontendTestFiles = testFiles.filter(
        (f) => f.endsWith(".ts") || f.endsWith(".tsx") || f.endsWith(".js") || f.endsWith(".jsx")
      );

      if (frontendTestFiles.length === 0) {
        console.log("No frontend test files found for changed files");
      } else {
        const result = await $`bun run test:coverage -- ${frontendTestFiles}`
          .cwd(projectRoot)
          .nothrow();

        if (result.exitCode !== 0) {
          exitCode = 1;
        }
      }
    } catch (error) {
      console.error("Frontend coverage failed:", error instanceof Error ? error.message : "Unknown error");
      exitCode = 1;
    }
  }

  if (target === "backend" || target === "both") {
    console.log("");
    console.log("=".repeat(60));
    console.log("              BACKEND COVERAGE (Debug)");
    console.log("=".repeat(60));
    console.log("");

    try {
      // Derive test modules from changed files
      const { deriveRustTestModule } = await import("../lib/utils");
      const testModules = changedFiles
        .map((f) => deriveRustTestModule(f))
        .filter((m): m is string => m !== null);
      const uniqueModules = [...new Set(testModules)];

      if (uniqueModules.length === 0) {
        console.log("No backend test modules found for changed files");
      } else {
        // Run cargo llvm-cov with text output for per-file details
        // --ignore-filename-regex excludes test files from coverage measurement
        const result = await $`cargo +nightly llvm-cov --ignore-filename-regex '_test\\.rs$'`
          .cwd(`${projectRoot}/src-tauri`)
          .nothrow();

        if (result.exitCode !== 0) {
          exitCode = 1;
        }
      }
    } catch (error) {
      console.error("Backend coverage failed:", error instanceof Error ? error.message : "Unknown error");
      exitCode = 1;
    }
  }

  process.exit(exitCode);
}
