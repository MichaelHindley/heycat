// ============================================================================
// Test Status Types
// ============================================================================

export const TEST_STATUSES = ["pass", "fail", "error", "skip"] as const;
export type TestStatus = (typeof TEST_STATUSES)[number];

// ============================================================================
// Test Target Types
// ============================================================================

export const TEST_TARGETS = ["frontend", "backend", "both"] as const;
export type TestTarget = (typeof TEST_TARGETS)[number];

// ============================================================================
// Constants
// ============================================================================

export const MAX_FAILURES = 5;
export const WIP_PREFIX = "WIP: ";
export const STATE_FILE = ".tcr-state.json";

// Output storage constants
export const OUTPUT_DIR = ".tcr/output";
export const TRUNCATE_SIZE = 5120;    // 5KB - always stored in state file
export const CHUNK_THRESHOLD = 10240; // 10KB - above this, write chunks
export const CHUNK_SIZE = 5120;       // 5KB chunks

// Frontend file extensions
export const FRONTEND_EXTENSIONS = [".ts", ".tsx", ".js", ".jsx"] as const;

// Backend file extensions
export const BACKEND_EXTENSIONS = [".rs"] as const;

// Report formatting
export const FORMATTING = {
  separatorWidth: 60,
} as const;

// ============================================================================
// State Interfaces
// ============================================================================

export interface TestOutput {
  truncated: string;           // First 5KB of combined output
  fullChunks: string[] | null; // Paths to chunk files if output > 10KB
  totalSize: number;           // Total output size in bytes
}

export interface TestResult {
  passed: boolean;
  timestamp: string;
  error: string | null;
  filesRun: string[];
  target: TestTarget;
  output?: TestOutput;         // Captured test output for debugging
}

export interface TCRState {
  currentStep: string | null;
  failureCount: number;
  lastTestResult: TestResult | null;
}

// ============================================================================
// Hook Input Interfaces
// ============================================================================

export interface HookInput {
  session_id: string;
  hook_event_name: string;
  tool_name: string;
  tool_input: Record<string, unknown>;
  tool_response?: Record<string, unknown>;
  cwd: string;
}

export interface TodoItem {
  content: string;
  status: "pending" | "in_progress" | "completed";
  activeForm: string;
}

export interface TodoWriteInput {
  todos: TodoItem[];
}

export interface BashInput {
  command: string;
  description?: string;
}

// ============================================================================
// Test Runner Interfaces
// ============================================================================

export interface TestRunResult {
  status: TestStatus;
  output: string;
  exitCode: number;
}
