import {
  loadState,
  setCurrentStep,
  incrementFailure,
  recordTestResult,
  hasReachedFailureThreshold,
} from "../lib/state";
import {
  findProjectRoot,
  getChangedFiles,
  findTestFiles,
  determineTarget,
  createWipCommit,
  getCurrentTimestamp,
} from "../lib/utils";
import {
  runTests,
  formatCondensedSuccess,
  formatCondensedFailure,
} from "../lib/test-runner";
import { MAX_FAILURES, type TestResult } from "../lib/types";

/**
 * Manual TCR check command
 *
 * Runs tests on changed files and auto-commits if they pass.
 * Unlike the hook, this is triggered explicitly by the user/agent.
 *
 * Usage:
 *   bun .claude/skills/tcr/tcr.ts check [step-name] [--verbose]
 *
 * Options:
 *   --verbose, -v  Show full test output (default: condensed on success)
 *
 * If step-name is omitted, defaults to "manual check"
 */
export async function handleCheck(args: string[]): Promise<void> {
  const projectRoot = await findProjectRoot();

  // Parse flags
  const verbose = args.includes("--verbose") || args.includes("-v");

  // Parse step name from args (everything that's not a flag)
  const stepId =
    args.filter((a) => !a.startsWith("-")).join(" ") || "manual check";

  console.log(`TCR: Running check - "${stepId}"`);

  // Update current step (resets failure count if step changed)
  await setCurrentStep(projectRoot, stepId);

  // Get changed files from git diff
  const changedFiles = await getChangedFiles(false);

  if (changedFiles.length === 0) {
    console.log("TCR: No changed files detected, nothing to test");
    process.exit(0);
  }

  console.log(`TCR: Found ${changedFiles.length} changed file(s)`);

  // Determine test target (frontend, backend, or both)
  const target = determineTarget(changedFiles);

  // Find test files for changed source files (frontend convention: foo.ts → foo.test.ts)
  const testFiles = await findTestFiles(changedFiles, projectRoot);

  // For frontend, we need discovered test files. For backend, tests are inline in source files.
  if (testFiles.length === 0 && target === "frontend") {
    // TCR principle: Don't auto-commit untested code
    console.warn("TCR: No test files found for changed files.");
    console.warn("  Changed files:", changedFiles.join(", "));
    console.warn("  Skipping auto-commit. Write tests first, or commit manually.");
    process.exit(0);
  }

  const testCount =
    target === "backend"
      ? changedFiles.filter((f) => f.endsWith(".rs")).length
      : testFiles.length;
  console.log(`TCR: Running ${target} tests (${testCount} file(s))...`);

  // Run tests - pass changed files for backend module filtering
  // Use quiet mode by default to reduce context window usage
  const result = await runTests(target, testFiles, projectRoot, changedFiles, {
    quiet: !verbose,
  });

  // Combine raw output from frontend and backend
  const rawOutputParts: string[] = [];
  if (result.frontend?.output) {
    rawOutputParts.push("=== Frontend Output ===\n" + result.frontend.output);
  }
  if (result.backend?.output) {
    rawOutputParts.push("=== Backend Output ===\n" + result.backend.output);
  }
  const rawOutput = rawOutputParts.join("\n\n");

  // Record test result with raw output
  const filesRun =
    target === "backend"
      ? changedFiles.filter((f) => f.endsWith(".rs"))
      : testFiles.length > 0
        ? testFiles
        : changedFiles;
  const testResult: TestResult = {
    passed: result.passed,
    timestamp: getCurrentTimestamp(),
    error: result.error,
    filesRun,
    target,
  };
  await recordTestResult(projectRoot, testResult, rawOutput);

  if (result.passed) {
    // Tests passed - show condensed output (or full if verbose)
    if (verbose) {
      console.log("TCR: Tests passed!");
      if (result.frontend?.output) console.log(result.frontend.output);
      if (result.backend?.output) console.log(result.backend.output);
    } else {
      console.log(`TCR: ${formatCondensedSuccess(result)}`);
    }

    const hash = await createWipCommit(stepId);
    if (hash) {
      console.log(`TCR: Committed (${hash})`);
    } else {
      console.log("TCR: No changes to commit");
    }
  } else {
    // Tests failed - increment failure counter with context and raw output
    const failureCount = await incrementFailure(
      projectRoot,
      result.error || "Tests failed",
      testFiles,
      target,
      rawOutput
    );

    // Get the saved output info to report chunk paths if applicable
    const state = await loadState(projectRoot);
    const savedOutput = state.lastTestResult?.output;
    const hasChunks = savedOutput?.fullChunks && savedOutput.fullChunks.length > 0;

    console.error(`TCR: Tests failed (${failureCount}/${MAX_FAILURES}) - details saved to .tcr-state.json`);

    if (hasChunks) {
      console.error(`TCR: Output exceeded 10KB, full output saved to .tcr/output/ (see .tcr-state.json for paths)`);
    }

    // Show error output (condensed or full based on verbose flag)
    if (verbose && result.error) {
      console.error(result.error);
    } else {
      console.error(formatCondensedFailure(result));
    }

    // Check if threshold reached
    if (hasReachedFailureThreshold(state)) {
      console.error("");
      console.error(`TCR: ${MAX_FAILURES} consecutive failures reached!`);
      console.error("Consider:");
      console.error("  1. Breaking down the task into smaller pieces");
      console.error("  2. Reviewing the test expectations");
      console.error("  3. Taking a different approach");
      console.error('  4. Run "bun .claude/skills/tcr/tcr.ts reset" to continue');
    }

    // Exit with code 2 to signal failure
    process.exit(2);
  }

  process.exit(0);
}
