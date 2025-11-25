import type { CoverageConfig, CoverageThresholds } from "./types";

// ============================================================================
// Coverage Configuration
// ============================================================================

/**
 * Centralized coverage configuration for both frontend and backend.
 * This is the single source of truth for coverage thresholds.
 *
 * Note: Frontend thresholds should match bunfig.toml for consistency.
 * Backend thresholds use cargo-llvm-cov --fail-under-lines/functions flags.
 */

export const DEFAULT_THRESHOLDS: CoverageThresholds = {
  lines: 0.8, // 80%
  functions: 0.8, // 80%
};

export const COVERAGE_CONFIG: Record<"frontend" | "backend", CoverageConfig> = {
  frontend: {
    enabled: true,
    thresholds: { ...DEFAULT_THRESHOLDS },
  },
  backend: {
    enabled: true,
    thresholds: { ...DEFAULT_THRESHOLDS },
  },
};

// ============================================================================
// Helper Functions
// ============================================================================

export function getThresholdPercentage(target: "frontend" | "backend", metric: "lines" | "functions"): number {
  return COVERAGE_CONFIG[target].thresholds[metric] * 100;
}

export function isTargetEnabled(target: "frontend" | "backend"): boolean {
  return COVERAGE_CONFIG[target].enabled;
}
