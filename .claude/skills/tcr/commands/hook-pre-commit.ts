import { loadState, recordTestResult } from "../lib/state";
import {
  findProjectRoot,
  getChangedFiles,
  findTestFiles,
  determineTarget,
  readStdin,
  getCurrentTimestamp,
} from "../lib/utils";
import { runTests } from "../lib/test-runner";
import type { HookInput, BashInput, TestResult } from "../lib/types";

/**
 * PreToolUse hook handler for Bash
 *
 * Intercepts git commit commands and blocks them if tests are failing.
 * Exit code 2 blocks the tool from executing.
 */
export async function handleHookPreCommit(): Promise<void> {
  try {
    // Read hook input from stdin
    const input = await readStdin<HookInput>();

    // Only handle Bash tool
    if (input.tool_name !== "Bash") {
      process.exit(0);
    }

    const bashInput = input.tool_input as unknown as BashInput;
    const command = bashInput.command || "";

    // Only intercept git commit commands
    if (!isCommitCommand(command)) {
      process.exit(0); // Allow non-commit commands
    }

    console.log("TCR: Pre-commit check triggered");

    const projectRoot = await findProjectRoot();

    // Get staged files
    const stagedFiles = await getChangedFiles(true);

    if (stagedFiles.length === 0) {
      // No staged files, allow commit (might be --allow-empty or amend)
      console.log("TCR: No staged files, allowing commit");
      process.exit(0);
    }

    // Find test files for staged source files
    const testFiles = await findTestFiles(stagedFiles, projectRoot);

    if (testFiles.length === 0) {
      // No tests to run, allow commit
      console.log("TCR: No tests found for staged files, allowing commit");
      process.exit(0);
    }

    // Determine test target
    const target = determineTarget(stagedFiles);
    console.log(`TCR: Running ${target} tests for ${testFiles.length} file(s)...`);

    // Run tests
    const result = await runTests(target, testFiles, projectRoot);

    // Record test result
    const testResult: TestResult = {
      passed: result.passed,
      timestamp: getCurrentTimestamp(),
      error: result.error,
      filesRun: testFiles,
      target,
    };
    await recordTestResult(projectRoot, testResult);

    if (result.passed) {
      console.log("TCR: Pre-commit tests passed ✅");
      process.exit(0); // Allow commit
    } else {
      // Block commit
      console.error("TCR: Pre-commit tests failed ❌");
      console.error("");

      if (result.error) {
        const errorPreview = result.error.slice(0, 500);
        console.error(errorPreview);
        if (result.error.length > 500) {
          console.error("...(truncated)");
        }
      }

      console.error("");
      console.error("Fix the failing tests before committing.");
      console.error('Run "bun .claude/skills/tcr/tcr.ts status" to see details.');

      // Exit code 2 blocks the tool
      process.exit(2);
    }
  } catch (error) {
    // Fail open - don't block commits on hook errors
    console.error("TCR hook error:", error instanceof Error ? error.message : "Unknown error");
    process.exit(0);
  }
}

/**
 * Check if a bash command is a git commit
 */
function isCommitCommand(command: string): boolean {
  // Match various git commit patterns
  const patterns = [
    /\bgit\s+commit\b/,
    /\bgit\s+.*\s+commit\b/, // git -C path commit
  ];

  return patterns.some((p) => p.test(command));
}
