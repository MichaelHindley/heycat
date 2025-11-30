import { readdir, readFile, unlink, stat } from "node:fs/promises";
import { join, basename } from "node:path";
import {
  TEMPLATES_DIR,
  SPEC_STATUSES,
  type Issue,
  type SpecInfo,
  type SpecFrontmatter,
  type SpecStatus,
  type ReviewHistoryEntry,
} from "./types";
import { validateSlug, toTitleCase, getCurrentDate } from "./utils";
import { validateSpecTransition } from "./validators/spec-status-validator";
import { hasReviewSection, getReviewVerdict, parseReviewSection, getFailedItems } from "./review-parser";

// ============================================================================
// Spec Manager Interface
// ============================================================================

export interface SpecManager {
  listSpecs(issue: Issue): Promise<SpecInfo[]>;
  getSpec(issue: Issue, specName: string): Promise<SpecInfo | null>;
  createSpec(projectRoot: string, issue: Issue, name: string, title?: string): Promise<SpecInfo>;
  updateStatus(spec: SpecInfo, status: SpecStatus): Promise<SpecInfo>;
  deleteSpec(spec: SpecInfo): Promise<void>;
  getCompletionStatus(issue: Issue): Promise<SpecCompletionStatus>;
  addReviewHistoryEntry(spec: SpecInfo, entry: ReviewHistoryEntry): Promise<SpecInfo>;
}

export interface SpecCompletionStatus {
  total: number;
  pending: number;
  inProgress: number;
  inReview: number;
  completed: number;
  allCompleted: boolean;
  specs: SpecInfo[];
}

// ============================================================================
// Spec Manager Implementation
// ============================================================================

export class FolderSpecManager implements SpecManager {
  /**
   * List all specs in an issue folder
   */
  async listSpecs(issue: Issue): Promise<SpecInfo[]> {
    const specs: SpecInfo[] = [];

    try {
      const entries = await readdir(issue.path);

      for (const entry of entries) {
        if (entry.endsWith(".spec.md")) {
          const specPath = join(issue.path, entry);
          const spec = await this.parseSpec(specPath);
          if (spec) {
            specs.push(spec);
          }
        }
      }
    } catch (err) {
      // Re-throw validation errors, ignore folder access errors
      if (err instanceof Error && err.message.includes("Invalid status")) {
        throw err;
      }
      // Issue folder might not exist or be accessible
    }

    // Sort by status (in-progress first, then in-review, then pending, then completed) and name
    return specs.sort((a, b) => {
      const statusOrder: Record<SpecStatus, number> = { "in-progress": 0, "in-review": 1, pending: 2, completed: 3 };
      const statusDiff = statusOrder[a.frontmatter.status] - statusOrder[b.frontmatter.status];
      if (statusDiff !== 0) return statusDiff;
      return a.name.localeCompare(b.name);
    });
  }

  /**
   * Get a specific spec by name
   */
  async getSpec(issue: Issue, specName: string): Promise<SpecInfo | null> {
    const slug = specName.replace(/\.spec\.md$/, "");
    const specPath = join(issue.path, `${slug}.spec.md`);

    try {
      await stat(specPath);
      return await this.parseSpec(specPath);
    } catch {
      return null;
    }
  }

  /**
   * Create a new spec in an issue folder
   */
  async createSpec(
    projectRoot: string,
    issue: Issue,
    name: string,
    title?: string
  ): Promise<SpecInfo> {
    validateSlug(name);

    const specPath = join(issue.path, `${name}.spec.md`);

    // Check if spec already exists
    try {
      await stat(specPath);
      throw new Error(`Spec "${name}" already exists in issue "${issue.name}"`);
    } catch (err) {
      if ((err as NodeJS.ErrnoException).code !== "ENOENT") {
        throw err;
      }
    }

    // Read spec template
    const templatePath = join(projectRoot, TEMPLATES_DIR, "spec.template.md");
    let content: string;

    try {
      content = await readFile(templatePath, "utf-8");
    } catch {
      throw new Error(`Spec template not found: ${templatePath}`);
    }

    const specTitle = title || toTitleCase(name);
    const date = getCurrentDate();

    content = content
      .replace(/\[Title\]/g, specTitle)
      .replace(/YYYY-MM-DD/g, date);

    await Bun.write(specPath, content);

    const spec = await this.parseSpec(specPath);
    if (!spec) {
      throw new Error("Failed to parse newly created spec");
    }

    return spec;
  }

  /**
   * Update a spec's status with validation
   */
  async updateStatus(spec: SpecInfo, status: SpecStatus): Promise<SpecInfo> {
    if (!SPEC_STATUSES.includes(status)) {
      throw new Error(`Invalid status: "${status}". Must be one of: ${SPEC_STATUSES.join(", ")}`);
    }

    const content = await readFile(spec.path, "utf-8");
    const date = getCurrentDate();

    // Validate the transition
    const hasReview = hasReviewSection(content);
    const verdict = hasReview ? getReviewVerdict(content) : null;

    const validationResult = validateSpecTransition({
      currentStatus: spec.frontmatter.status,
      targetStatus: status,
      hasReviewSection: hasReview,
      reviewVerdict: verdict,
    });

    if (!validationResult.valid) {
      throw new Error(validationResult.error);
    }

    // Update frontmatter
    let updatedContent = content;

    // Update status
    updatedContent = updatedContent.replace(
      /^status:\s*.+$/m,
      `status: ${status}`
    );

    // Update completed date if completing
    if (status === "completed") {
      updatedContent = updatedContent.replace(
        /^completed:\s*.+$/m,
        `completed: ${date}`
      );
    } else {
      updatedContent = updatedContent.replace(
        /^completed:\s*.+$/m,
        "completed: null"
      );
    }

    // Increment review_round when entering in-review
    if (status === "in-review" && spec.frontmatter.status !== "in-review") {
      const newRound = (spec.frontmatter.review_round || 0) + 1;
      if (updatedContent.match(/^review_round:\s*\d+/m)) {
        updatedContent = updatedContent.replace(
          /^review_round:\s*\d+/m,
          `review_round: ${newRound}`
        );
      } else {
        // Add review_round after dependencies line
        updatedContent = updatedContent.replace(
          /^(dependencies:\s*\[.*\])$/m,
          `$1\nreview_round: ${newRound}`
        );
      }
    }

    await Bun.write(spec.path, updatedContent);

    // Return updated spec info
    const updatedSpec = await this.parseSpec(spec.path);
    if (!updatedSpec) {
      throw new Error("Failed to parse updated spec");
    }

    return updatedSpec;
  }

  /**
   * Delete a spec
   */
  async deleteSpec(spec: SpecInfo): Promise<void> {
    await unlink(spec.path);
  }

  /**
   * Get completion status summary for an issue
   */
  async getCompletionStatus(issue: Issue): Promise<SpecCompletionStatus> {
    const specs = await this.listSpecs(issue);

    const pending = specs.filter((s) => s.frontmatter.status === "pending").length;
    const inProgress = specs.filter((s) => s.frontmatter.status === "in-progress").length;
    const inReview = specs.filter((s) => s.frontmatter.status === "in-review").length;
    const completed = specs.filter((s) => s.frontmatter.status === "completed").length;

    return {
      total: specs.length,
      pending,
      inProgress,
      inReview,
      completed,
      allCompleted: specs.length > 0 && pending === 0 && inProgress === 0 && inReview === 0,
      specs,
    };
  }

  /**
   * Add a review history entry to a spec
   */
  async addReviewHistoryEntry(spec: SpecInfo, entry: ReviewHistoryEntry): Promise<SpecInfo> {
    const content = await readFile(spec.path, "utf-8");
    let updatedContent = content;

    const currentHistory = spec.frontmatter.review_history || [];
    const newHistory = [...currentHistory, entry];

    // Serialize review_history as YAML array
    const historyYaml = this.serializeReviewHistory(newHistory);

    if (updatedContent.match(/^review_history:/m)) {
      // Replace existing review_history
      updatedContent = updatedContent.replace(
        /^review_history:[\s\S]*?(?=^[a-z_]+:|^---$)/m,
        historyYaml
      );
    } else {
      // Add review_history after review_round or dependencies
      if (updatedContent.match(/^review_round:/m)) {
        updatedContent = updatedContent.replace(
          /^(review_round:\s*\d+)$/m,
          `$1\n${historyYaml.trim()}`
        );
      } else {
        updatedContent = updatedContent.replace(
          /^(dependencies:\s*\[.*\])$/m,
          `$1\n${historyYaml.trim()}`
        );
      }
    }

    await Bun.write(spec.path, updatedContent);

    const updatedSpec = await this.parseSpec(spec.path);
    if (!updatedSpec) {
      throw new Error("Failed to parse updated spec");
    }

    return updatedSpec;
  }

  /**
   * Serialize review history to YAML format
   */
  private serializeReviewHistory(history: ReviewHistoryEntry[]): string {
    if (history.length === 0) return "review_history: []\n";

    const entries = history.map((entry) => {
      const failedCriteria = entry.failedCriteria.length > 0
        ? `[${entry.failedCriteria.map(c => `"${c.replace(/"/g, '\\"')}"`).join(", ")}]`
        : "[]";
      const concerns = entry.concerns.length > 0
        ? `[${entry.concerns.map(c => `"${c.replace(/"/g, '\\"')}"`).join(", ")}]`
        : "[]";

      return `  - round: ${entry.round}
    date: ${entry.date}
    verdict: ${entry.verdict}
    failedCriteria: ${failedCriteria}
    concerns: ${concerns}`;
    });

    return `review_history:\n${entries.join("\n")}\n`;
  }

  // ============================================================================
  // Private Helpers
  // ============================================================================

  /**
   * Parse a spec file into SpecInfo
   */
  private async parseSpec(specPath: string): Promise<SpecInfo | null> {
    try {
      const content = await readFile(specPath, "utf-8");
      const name = basename(specPath).replace(/\.spec\.md$/, "");

      // Parse YAML frontmatter - may throw on invalid values
      const frontmatter = this.parseFrontmatter(content);

      // Parse title from content
      const titleMatch = content.match(/^#\s+Spec:\s*(.+)$/m);
      const title = titleMatch ? titleMatch[1].trim() : toTitleCase(name);

      return {
        name,
        path: specPath,
        frontmatter,
        title,
      };
    } catch (err) {
      // Re-throw validation errors with file context
      if (err instanceof Error && err.message.includes("Invalid status")) {
        throw new Error(`${specPath}: ${err.message}`);
      }
      // Other errors (file not found, etc.) return null
      return null;
    }
  }

  /**
   * Parse YAML frontmatter from spec content
   */
  private parseFrontmatter(content: string): SpecFrontmatter {
    const defaults: SpecFrontmatter = {
      status: "pending",
      created: getCurrentDate(),
      completed: null,
      dependencies: [],
    };

    // Match frontmatter block
    const fmMatch = content.match(/^---\n([\s\S]*?)\n---/);
    if (!fmMatch) return defaults;

    const fmContent = fmMatch[1];

    // Parse status
    const statusMatch = fmContent.match(/^status:\s*(.+)$/m);
    if (statusMatch) {
      const status = statusMatch[1].trim();
      if (SPEC_STATUSES.includes(status as SpecStatus)) {
        defaults.status = status as SpecStatus;
      } else {
        throw new Error(
          `Invalid status "${status}". Valid statuses: ${SPEC_STATUSES.join(", ")}`
        );
      }
    }

    // Parse created
    const createdMatch = fmContent.match(/^created:\s*(\d{4}-\d{2}-\d{2})/m);
    if (createdMatch) {
      defaults.created = createdMatch[1];
    }

    // Parse completed
    const completedMatch = fmContent.match(/^completed:\s*(\d{4}-\d{2}-\d{2}|null)/m);
    if (completedMatch && completedMatch[1] !== "null") {
      defaults.completed = completedMatch[1];
    }

    // Parse dependencies (YAML array)
    const depsMatch = fmContent.match(/^dependencies:\s*\[(.*)\]/m);
    if (depsMatch && depsMatch[1].trim()) {
      defaults.dependencies = depsMatch[1]
        .split(",")
        .map((d) => d.trim().replace(/['"]/g, ""))
        .filter(Boolean);
    }

    // Parse review_round (optional)
    const reviewRoundMatch = fmContent.match(/^review_round:\s*(\d+)/m);
    if (reviewRoundMatch) {
      defaults.review_round = parseInt(reviewRoundMatch[1], 10);
    }

    // Note: review_history is complex YAML and would require a full parser
    // For now, we parse it only when explicitly needed via the review-parser module

    return defaults;
  }
}

// ============================================================================
// Default Export
// ============================================================================

export function createSpecManager(): SpecManager {
  return new FolderSpecManager();
}
