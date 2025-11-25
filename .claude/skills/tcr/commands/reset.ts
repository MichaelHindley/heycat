import { resetFailures, loadState } from "../lib/state";
import { findProjectRoot } from "../lib/utils";

export async function handleReset(_args: string[]): Promise<void> {
  const projectRoot = await findProjectRoot();
  const state = await loadState(projectRoot);

  const previousCount = state.failureCount;

  await resetFailures(projectRoot);

  if (previousCount > 0) {
    console.log(`TCR: Failure counter reset (was ${previousCount})`);
    console.log("You can continue working on the current task.");
  } else {
    console.log("TCR: Failure counter was already at 0");
  }
}
