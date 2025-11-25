import type { CoverageResult, CoverageMetrics } from "../types";
import { createEmptyMetrics, meetsThresholds } from "../types";
import { COVERAGE_CONFIG } from "../config";

// ============================================================================
// Bun Coverage Output Parser
// ============================================================================

/**
 * Parse Bun's coverage output to extract metrics.
 * Bun outputs coverage in a table format like:
 *
 * ------------------|---------|----------|---------|---------|
 * File              | % Funcs | % Lines  | Uncovered Lines
 * ------------------|---------|----------|---------|---------|
 * All files         |   85.71 |    90.00 |
 * src/example.ts    |   85.71 |    90.00 | 12-15
 * ------------------|---------|----------|---------|---------|
 */
function parseBunCoverageOutput(output: string): CoverageMetrics {
  try {
    // Look for "All files" line which contains aggregate coverage
    const allFilesMatch = output.match(/All files\s*\|\s*([\d.]+)\s*\|\s*([\d.]+)/);

    if (allFilesMatch) {
      const funcsPercent = parseFloat(allFilesMatch[1]) / 100;
      const linesPercent = parseFloat(allFilesMatch[2]) / 100;

      return {
        lines: {
          covered: 0, // Bun doesn't provide raw counts in table output
          total: 0,
          percentage: linesPercent,
        },
        functions: {
          covered: 0,
          total: 0,
          percentage: funcsPercent,
        },
      };
    }

    // Fallback: if we can't parse, return empty metrics
    return createEmptyMetrics();
  } catch {
    return createEmptyMetrics();
  }
}

// ============================================================================
// Frontend Coverage Runner
// ============================================================================

export async function runFrontendCoverage(
  testFiles: string[],
  projectRoot: string
): Promise<CoverageResult> {
  const { $ } = await import("bun");
  const config = COVERAGE_CONFIG.frontend;

  // If no test files, return skip with passing status
  if (testFiles.length === 0) {
    return {
      target: "frontend",
      passed: true,
      metrics: {
        lines: { covered: 0, total: 0, percentage: 1 },
        functions: { covered: 0, total: 0, percentage: 1 },
      },
      thresholds: config.thresholds,
      raw: "No frontend test files to run",
    };
  }

  try {
    // Run bun test with coverage
    // bunfig.toml enforces thresholds, so exit code indicates pass/fail
    const result = await $`bun test --coverage ${testFiles}`.cwd(projectRoot).quiet().nothrow();

    const stdout = result.stdout.toString();
    const stderr = result.stderr.toString();
    const output = stdout + stderr;

    // Parse coverage metrics from output
    const metrics = parseBunCoverageOutput(output);

    // Bun enforces thresholds via bunfig.toml, exit code 0 = pass
    const passed = result.exitCode === 0;

    return {
      target: "frontend",
      passed,
      metrics,
      thresholds: config.thresholds,
      raw: output,
      error: passed ? undefined : "Tests failed or coverage below threshold",
    };
  } catch (error) {
    return {
      target: "frontend",
      passed: false,
      metrics: createEmptyMetrics(),
      thresholds: config.thresholds,
      error: error instanceof Error ? error.message : "Unknown error running frontend coverage",
    };
  }
}

/**
 * Run frontend coverage check for all tests (not specific files).
 * Used by the coverage command and status display.
 */
export async function runFrontendCoverageAll(projectRoot: string): Promise<CoverageResult> {
  const { $ } = await import("bun");
  const config = COVERAGE_CONFIG.frontend;

  try {
    const result = await $`bun test --coverage`.cwd(projectRoot).quiet().nothrow();

    const stdout = result.stdout.toString();
    const stderr = result.stderr.toString();
    const output = stdout + stderr;

    const metrics = parseBunCoverageOutput(output);
    const passed = result.exitCode === 0;

    return {
      target: "frontend",
      passed,
      metrics,
      thresholds: config.thresholds,
      raw: output,
      error: passed ? undefined : "Tests failed or coverage below threshold",
    };
  } catch (error) {
    return {
      target: "frontend",
      passed: false,
      metrics: createEmptyMetrics(),
      thresholds: config.thresholds,
      error: error instanceof Error ? error.message : "Unknown error running frontend coverage",
    };
  }
}
