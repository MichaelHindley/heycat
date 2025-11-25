import { readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { STATE_FILE, MAX_FAILURES, type TCRState, type TestResult } from "./types";
import { fileExists, getCurrentTimestamp } from "./utils";

// ============================================================================
// Default State
// ============================================================================

function createDefaultState(): TCRState {
  return {
    currentStep: null,
    failureCount: 0,
    lastTestResult: null,
  };
}

// ============================================================================
// State Persistence
// ============================================================================

export async function loadState(projectRoot: string): Promise<TCRState> {
  const statePath = join(projectRoot, STATE_FILE);

  try {
    if (await fileExists(statePath)) {
      const content = await readFile(statePath, "utf-8");
      return JSON.parse(content) as TCRState;
    }
  } catch {
    // If parsing fails, return default state
  }

  return createDefaultState();
}

export async function saveState(projectRoot: string, state: TCRState): Promise<void> {
  const statePath = join(projectRoot, STATE_FILE);
  await writeFile(statePath, JSON.stringify(state, null, 2), "utf-8");
}

// ============================================================================
// State Mutations
// ============================================================================

export async function setCurrentStep(projectRoot: string, step: string): Promise<void> {
  const state = await loadState(projectRoot);

  // If step changed, reset failure count
  if (state.currentStep !== step) {
    state.currentStep = step;
    state.failureCount = 0;
  }

  await saveState(projectRoot, state);
}

export async function incrementFailure(
  projectRoot: string,
  error: string
): Promise<number> {
  const state = await loadState(projectRoot);

  state.failureCount += 1;
  state.lastTestResult = {
    passed: false,
    timestamp: getCurrentTimestamp(),
    error,
    filesRun: [],
    target: "frontend",
  };

  await saveState(projectRoot, state);
  return state.failureCount;
}

export async function resetFailures(projectRoot: string): Promise<void> {
  const state = await loadState(projectRoot);
  state.failureCount = 0;
  await saveState(projectRoot, state);
}

export async function recordTestResult(
  projectRoot: string,
  result: TestResult
): Promise<void> {
  const state = await loadState(projectRoot);

  state.lastTestResult = result;

  // Reset failure count on success
  if (result.passed) {
    state.failureCount = 0;
  }

  await saveState(projectRoot, state);
}

// ============================================================================
// State Queries
// ============================================================================

export function hasReachedFailureThreshold(state: TCRState): boolean {
  return state.failureCount >= MAX_FAILURES;
}

export function getFailureCount(state: TCRState): number {
  return state.failureCount;
}
