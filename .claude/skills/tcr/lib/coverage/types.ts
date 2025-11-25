// ============================================================================
// Coverage Threshold Types
// ============================================================================

export interface CoverageThresholds {
  lines: number; // 0.0 - 1.0
  functions: number; // 0.0 - 1.0
}

export interface CoverageConfig {
  enabled: boolean;
  thresholds: CoverageThresholds;
}

// ============================================================================
// Coverage Metrics Types
// ============================================================================

export interface CoverageMetricDetail {
  covered: number;
  total: number;
  percentage: number; // 0.0 - 1.0
}

export interface CoverageMetrics {
  lines: CoverageMetricDetail;
  functions: CoverageMetricDetail;
}

// ============================================================================
// Coverage Result Types
// ============================================================================

export type CoverageTarget = "frontend" | "backend";

export interface CoverageResult {
  target: CoverageTarget;
  passed: boolean;
  metrics: CoverageMetrics;
  thresholds: CoverageThresholds;
  raw?: string;
  error?: string;
}

export interface CombinedCoverageResult {
  passed: boolean;
  frontend: CoverageResult | null;
  backend: CoverageResult | null;
  summary: string;
}

// ============================================================================
// Utility Functions
// ============================================================================

export function createEmptyMetrics(): CoverageMetrics {
  return {
    lines: { covered: 0, total: 0, percentage: 0 },
    functions: { covered: 0, total: 0, percentage: 0 },
  };
}

export function meetsThresholds(metrics: CoverageMetrics, thresholds: CoverageThresholds): boolean {
  return metrics.lines.percentage >= thresholds.lines && metrics.functions.percentage >= thresholds.functions;
}

export function formatPercentage(value: number): string {
  return `${(value * 100).toFixed(1)}%`;
}
