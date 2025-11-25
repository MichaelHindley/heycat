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
  readStdin,
  getCurrentTimestamp,
} from "../lib/utils";
import { runTests } from "../lib/test-runner";
import {
  MAX_FAILURES,
  type HookInput,
  type TodoWriteInput,
  type TestResult,
} from "../lib/types";

/**
 * PostToolUse hook handler for TodoWrite
 *
 * Triggered when TodoWrite tool completes. If a todo was marked "completed",
 * runs tests on changed files and auto-commits if they pass.
 */
export async function handleHookTodoComplete(): Promise<void> {
  try {
    // Read hook input from stdin
    const input = await readStdin<HookInput>();

    // Verify this is a TodoWrite event
    if (input.tool_name !== "TodoWrite") {
      process.exit(0);
    }

    const todoInput = input.tool_input as unknown as TodoWriteInput;
    const projectRoot = await findProjectRoot();

    // Find newly completed todos
    const completedTodos = todoInput.todos.filter((t) => t.status === "completed");

    if (completedTodos.length === 0) {
      // No completed todos, nothing to do
      process.exit(0);
    }

    // Get the most recently completed todo (last in list)
    const completedTodo = completedTodos[completedTodos.length - 1];
    const stepId = completedTodo.content;

    console.log(`TCR: Todo completed - "${stepId}"`);

    // Update current step (resets failure count if step changed)
    await setCurrentStep(projectRoot, stepId);

    // Get changed files from git diff
    const changedFiles = await getChangedFiles(false);

    if (changedFiles.length === 0) {
      console.log("TCR: No changed files detected, skipping tests");
      process.exit(0);
    }

    console.log(`TCR: Found ${changedFiles.length} changed file(s)`);

    // Find test files for changed source files
    const testFiles = await findTestFiles(changedFiles, projectRoot);

    if (testFiles.length === 0) {
      console.log("TCR: No test files found for changed files");
      // Auto-commit WIP even without tests
      const hash = await createWipCommit(stepId);
      if (hash) {
        console.log(`TCR: Created WIP commit (${hash})`);
      }
      process.exit(0);
    }

    // Determine test target (frontend, backend, or both)
    const target = determineTarget(changedFiles);
    console.log(`TCR: Running ${target} tests (${testFiles.length} file(s))...`);

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
      // Tests passed - create WIP commit
      console.log("TCR: Tests passed!");

      const hash = await createWipCommit(stepId);
      if (hash) {
        console.log(`TCR: Created WIP commit (${hash})`);
      }
    } else {
      // Tests failed - increment failure counter
      const failureCount = await incrementFailure(projectRoot, result.error || "Tests failed");

      console.error(`TCR: Tests failed (${failureCount}/${MAX_FAILURES})`);

      // Show truncated error output
      if (result.error) {
        const errorPreview = result.error.slice(0, 300);
        console.error(errorPreview);
        if (result.error.length > 300) {
          console.error("...(truncated)");
        }
      }

      // Check if threshold reached
      const state = await loadState(projectRoot);
      if (hasReachedFailureThreshold(state)) {
        console.error("");
        console.error(`TCR: ⚠️  ${MAX_FAILURES} consecutive failures reached!`);
        console.error("Consider:");
        console.error("  1. Breaking down the task into smaller pieces");
        console.error("  2. Reviewing the test expectations");
        console.error("  3. Taking a different approach");
        console.error('  4. Run "bun .claude/skills/tcr/tcr.ts reset" to continue');
      }
    }

    process.exit(0);
  } catch (error) {
    // Fail open - don't block Claude Code on hook errors
    console.error("TCR hook error:", error instanceof Error ? error.message : "Unknown error");
    process.exit(0);
  }
}
