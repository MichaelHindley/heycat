import {
  type Stage,
  type Issue,
  type IssueAnalysis,
  type ValidationResult,
} from "../types";
import type { TransitionValidator } from "./validator-chain";

/**
 * Validates that features have BDD scenarios defined before moving to todo.
 * This ensures product research (Given/When/Then scenarios) is done before
 * breaking down into specs.
 *
 * Applies to: 2-todo (backlog -> todo transition)
 * Only enforced for: features (bugs and tasks skip this validation)
 */
export class BDDScenariosValidator implements TransitionValidator {
  readonly name = "BDDScenariosValidator";
  readonly appliesTo: Stage[] = ["2-todo"];

  validate(issue: Issue, analysis: IssueAnalysis, _toStage: Stage): ValidationResult {
    // Only applies to features - bugs and tasks don't need BDD scenarios
    if (issue.type !== "feature") {
      return { valid: true, missing: [] };
    }

    if (!analysis.hasBDDScenarios) {
      return {
        valid: false,
        missing: [
          "BDD Scenarios required for features. Run 'agile.ts discover <name>' for guided " +
          "scenario creation, or manually add Given/When/Then scenarios to the BDD Scenarios section.",
        ],
      };
    }

    return { valid: true, missing: [] };
  }
}
