import { loadState } from "../lib/state";
import { findProjectRoot } from "../lib/utils";
import { MAX_FAILURES } from "../lib/types";

export async function handleStatus(_args: string[]): Promise<void> {
  const projectRoot = await findProjectRoot();
  const state = await loadState(projectRoot);

  console.log("\n=== TCR Status ===\n");

  // Current step
  if (state.currentStep) {
    console.log(`Current Step: ${state.currentStep}`);
  } else {
    console.log("Current Step: None (no active task)");
  }

  // Failure count
  const failureBar = "█".repeat(state.failureCount) + "░".repeat(MAX_FAILURES - state.failureCount);
  console.log(`Failures: [${failureBar}] ${state.failureCount}/${MAX_FAILURES}`);

  if (state.failureCount >= MAX_FAILURES) {
    console.log("  ⚠️  Threshold reached - consider a different approach");
    console.log('  Run "tcr reset" to continue');
  }

  // Last test result
  console.log("");
  if (state.lastTestResult) {
    const { passed, timestamp, error, target } = state.lastTestResult;
    const status = passed ? "✅ PASS" : "❌ FAIL";
    const time = new Date(timestamp).toLocaleString();

    console.log(`Last Test: ${status}`);
    console.log(`  Time: ${time}`);
    console.log(`  Target: ${target}`);

    if (error && !passed) {
      console.log(`  Error: ${error.slice(0, 200)}${error.length > 200 ? "..." : ""}`);
    }
  } else {
    console.log("Last Test: No tests run yet");
  }

  console.log("");
}
