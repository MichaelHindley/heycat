import { readFile, writeFile, mkdir, readdir, unlink } from "node:fs/promises";
import { join } from "node:path";
import {
  STATE_FILE,
  MAX_FAILURES,
  TEST_TARGETS,
  OUTPUT_DIR,
  TRUNCATE_SIZE,
  CHUNK_THRESHOLD,
  CHUNK_SIZE,
  type TCRState,
  type TestResult,
  type TestTarget,
  type TestOutput,
} from "./types";
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
// State Validation
// ============================================================================

/**
 * Validate that a parsed object conforms to TCRState shape.
 * Returns true if valid, false if corrupted or wrong format.
 */
function isValidState(obj: unknown): obj is TCRState {
  if (typeof obj !== "object" || obj === null) {
    return false;
  }

  const state = obj as Record<string, unknown>;

  // Check required fields exist and have correct types
  if (!("currentStep" in state) || !("failureCount" in state)) {
    return false;
  }

  // currentStep should be string or null
  if (state.currentStep !== null && typeof state.currentStep !== "string") {
    return false;
  }

  // failureCount should be a non-negative number
  if (typeof state.failureCount !== "number" || state.failureCount < 0) {
    return false;
  }

  // Validate lastTestResult if present
  if (state.lastTestResult !== null && state.lastTestResult !== undefined) {
    const result = state.lastTestResult as Record<string, unknown>;

    // Check required TestResult fields
    if (typeof result.passed !== "boolean") return false;
    if (typeof result.timestamp !== "string") return false;
    if (!TEST_TARGETS.includes(result.target as TestTarget)) return false;
    if (!Array.isArray(result.filesRun)) return false;
    // error can be string or null
    if (result.error !== null && typeof result.error !== "string") return false;

    // Validate output if present (optional field)
    if (result.output !== undefined) {
      const output = result.output as Record<string, unknown>;
      if (typeof output.truncated !== "string") return false;
      if (typeof output.totalSize !== "number") return false;
      // fullChunks can be null or array of strings
      if (output.fullChunks !== null) {
        if (!Array.isArray(output.fullChunks)) return false;
        if (!output.fullChunks.every((c) => typeof c === "string")) return false;
      }
    }
  }

  return true;
}

// ============================================================================
// State Persistence
// ============================================================================

export async function loadState(projectRoot: string): Promise<TCRState> {
  const statePath = join(projectRoot, STATE_FILE);

  try {
    if (await fileExists(statePath)) {
      const content = await readFile(statePath, "utf-8");
      const parsed = JSON.parse(content);

      if (!isValidState(parsed)) {
        console.warn("TCR: Invalid state file, resetting to defaults");
        return createDefaultState();
      }

      return parsed;
    }
  } catch (error) {
    console.warn(
      "TCR: Error loading state file:",
      error instanceof Error ? error.message : "Unknown error"
    );
  }

  return createDefaultState();
}

export async function saveState(projectRoot: string, state: TCRState): Promise<void> {
  const statePath = join(projectRoot, STATE_FILE);
  await writeFile(statePath, JSON.stringify(state, null, 2), "utf-8");
}

// ============================================================================
// Output Persistence
// ============================================================================

/**
 * Ensure the output directory exists for storing chunk files.
 */
async function ensureOutputDir(projectRoot: string): Promise<string> {
  const outputPath = join(projectRoot, OUTPUT_DIR);
  await mkdir(outputPath, { recursive: true });
  return outputPath;
}

/**
 * Remove all existing chunk files from the output directory.
 */
async function cleanupOldChunks(projectRoot: string): Promise<void> {
  const outputPath = join(projectRoot, OUTPUT_DIR);

  try {
    if (!(await fileExists(outputPath))) return;

    const files = await readdir(outputPath);
    const chunkFiles = files.filter((f) => f.startsWith("chunk-"));

    await Promise.all(
      chunkFiles.map((f) => unlink(join(outputPath, f)))
    );
  } catch {
    // Ignore cleanup errors - not critical
  }
}

/**
 * Save test output with truncation and chunking.
 *
 * Strategy:
 * - Always store first 5KB in truncated field
 * - If output > 10KB, split full output into 5KB chunks and save to .tcr/output/
 * - Return TestOutput with paths to any chunk files
 */
export async function saveTestOutput(
  projectRoot: string,
  output: string
): Promise<TestOutput> {
  const totalSize = Buffer.byteLength(output, "utf-8");
  const truncated = output.slice(0, TRUNCATE_SIZE);

  // If output is small enough, no need for chunks
  if (totalSize <= CHUNK_THRESHOLD) {
    return {
      truncated,
      fullChunks: null,
      totalSize,
    };
  }

  // Output exceeds threshold - write chunks
  const outputDir = await ensureOutputDir(projectRoot);
  await cleanupOldChunks(projectRoot);

  const timestamp = Date.now();
  const chunks: string[] = [];

  // Split output into chunks
  let offset = 0;
  let index = 0;
  while (offset < output.length) {
    const chunk = output.slice(offset, offset + CHUNK_SIZE);
    const chunkName = `chunk-${timestamp}-${index}.txt`;
    const chunkPath = join(outputDir, chunkName);

    await writeFile(chunkPath, chunk, "utf-8");
    chunks.push(join(OUTPUT_DIR, chunkName)); // Store relative path

    offset += CHUNK_SIZE;
    index++;
  }

  return {
    truncated,
    fullChunks: chunks,
    totalSize,
  };
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
  error: string,
  filesRun: string[] = [],
  target: TestTarget = "frontend",
  rawOutput?: string
): Promise<number> {
  const state = await loadState(projectRoot);

  // Save output if provided
  let output: TestOutput | undefined;
  if (rawOutput) {
    output = await saveTestOutput(projectRoot, rawOutput);
  }

  state.failureCount += 1;
  state.lastTestResult = {
    passed: false,
    timestamp: getCurrentTimestamp(),
    error,
    filesRun,
    target,
    output,
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
  result: TestResult,
  rawOutput?: string
): Promise<void> {
  const state = await loadState(projectRoot);

  // Save output if provided and not already in result
  if (rawOutput && !result.output) {
    result.output = await saveTestOutput(projectRoot, rawOutput);
  }

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
