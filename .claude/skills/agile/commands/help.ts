import { STAGES, STAGE_NAMES, SPEC_STATUSES, GUIDANCE_STATUSES } from "../lib/types";

export function handleHelp(args: string[]): void {
  const [command] = args;

  if (command) {
    switch (command) {
      case "create":
        console.log(`
Usage: agile.ts create <type> <name> [options]

Create a new folder-based issue from a template.

Arguments:
  type     Issue type: feature, bug, or task
  name     Kebab-case name for the issue (e.g., user-authentication)

Options:
  --title, -t    Human-readable title (defaults to name in Title Case)
  --owner, -o    Issue owner/assignee name
  --stage, -s    Initial stage (default: 1-backlog)

Creates:
  agile/<stage>/<name>/
    - <type>.md              Main issue spec
    - technical-guidance.md  Technical investigation document

Examples:
  agile.ts create feature user-auth --title "User Authentication" --owner "Alice"
  agile.ts create bug fix-login --stage 2-todo --owner "Bob"
`);
        break;
      case "move":
        console.log(`
Usage: agile.ts move <name> <stage>

Move an issue folder to a different workflow stage.

Arguments:
  name     Issue name
  stage    Target stage: ${STAGES.join(", ")}

Workflow (only sequential transitions allowed):
  1-backlog -> 2-todo -> 3-in-progress -> 4-review -> 5-done

Validation Requirements:
  - 2-todo: Description complete, BDD scenarios defined (features only)
  - 3-in-progress: Owner assigned, technical guidance exists
  - 4-review: All specs completed, guidance updated
  - 5-done: All Definition of Done items checked

BDD Scenarios (Features Only):
  Features require Given/When/Then scenarios before moving to todo.
  Run 'agile.ts discover <name>' for guided scenario creation.

Examples:
  agile.ts move user-auth 2-todo
  agile.ts move fix-login 3-in-progress
`);
        break;
      case "list":
        console.log(`
Usage: agile.ts list [options]

List all issues or filter by stage.

Options:
  --stage, -s     Filter by stage
  --format, -f    Output format: table (default) or json

Examples:
  agile.ts list
  agile.ts list --stage 3-in-progress
  agile.ts list --format json
`);
        break;
      case "archive":
        console.log(`
Usage: agile.ts archive <name>

Archive an issue folder (move to agile/archive/<name>-<date>/).

Arguments:
  name     Issue name to archive

Examples:
  agile.ts archive completed-feature
`);
        break;
      case "delete":
        console.log(`
Usage: agile.ts delete <name>

Permanently delete an issue folder and all its specs.

Arguments:
  name     Issue name to delete

Examples:
  agile.ts delete old-task
`);
        break;
      case "work":
        console.log(`
Usage: agile.ts work <name>

Analyze an issue and get stage-appropriate guidance.

Arguments:
  name     Issue name to analyze

Output includes:
  - Issue metadata (type, stage, owner, created date)
  - Specs status (pending, in-progress, completed)
  - Technical guidance status
  - Incomplete sections with placeholder text
  - Definition of Done progress
  - Stage-specific guidance and suggested actions
  - Readiness status for advancing to the next stage

Examples:
  agile.ts work user-auth
  agile.ts work dark-mode
`);
        break;
      case "spec":
        console.log(`
Usage: agile.ts spec <subcommand> [options]

Manage specs within an issue folder.

Subcommands:
  list <issue>                          List all specs in an issue
  add <issue> <name> [--title "..."]    Add a new spec
  status <issue> <spec> <status>        Update spec status
  delete <issue> <spec>                 Delete a spec
  suggest <issue>                       AI-assisted spec breakdown

Statuses: ${SPEC_STATUSES.join(", ")}

Note: Completing a spec requires technical guidance to be updated first.

Examples:
  agile.ts spec list user-auth
  agile.ts spec add user-auth login-flow --title "Implement Login Flow"
  agile.ts spec status user-auth login-flow in-progress
  agile.ts spec status user-auth login-flow completed
  agile.ts spec delete user-auth unused-spec
  agile.ts spec suggest user-auth
`);
        break;
      case "guidance":
        console.log(`
Usage: agile.ts guidance <subcommand> [options]

Manage technical guidance for an issue.

Subcommands:
  show <issue>                  Show technical guidance summary
  update <issue>                Mark guidance as updated (set timestamp)
  validate <issue>              Check if guidance is current
  status <issue> <status>       Set guidance status

Statuses: ${GUIDANCE_STATUSES.join(", ")}

Technical guidance must be updated before completing specs.

Examples:
  agile.ts guidance show user-auth
  agile.ts guidance update user-auth
  agile.ts guidance validate user-auth
  agile.ts guidance status user-auth active
`);
        break;
      case "review":
        console.log(`
Usage: agile.ts review

Review a spec in in-review status in the current in-progress issue.

The command:
  1. Finds the issue in 3-in-progress (must be exactly one)
  2. Finds specs with in-review status
  3. Uses git history to identify the most recently modified in-review spec
  4. Outputs a review prompt for Claude to execute

The review covers:
  - Implementation verification (acceptance criteria vs actual code)
  - Code quality audit (patterns, error handling, test coverage)

Output:
  A structured prompt that guides Claude to:
  - Read implementation files referenced in the spec
  - Verify each acceptance criterion is implemented
  - Check test coverage matches test cases
  - Generate and append a Review section to the spec file

After Review:
  - If APPROVED: Run spec status <issue> <spec> completed
  - If NEEDS_WORK: Run agile.ts fix to address feedback

Examples:
  agile.ts review
`);
        break;
      case "fix":
        console.log(`
Usage: agile.ts fix

Fix a spec that received NEEDS_WORK verdict during review.

The command:
  1. Finds spec in in-review status with NEEDS_WORK verdict
  2. Parses the Review section to extract:
     - Failed acceptance criteria with evidence
     - Missing test coverage
     - Code quality concerns
  3. Displays a structured fix guide
  4. Transitions spec back to in-progress

After Fixing:
  Run /agile:review for another independent review round.

Review History:
  Each review round is tracked in the spec frontmatter:
  - review_round: Current iteration number
  - review_history: Array of past review verdicts

Examples:
  agile.ts fix
`);
        break;
      case "discover":
        console.log(`
Usage: agile.ts discover <name>

Guide BDD scenario creation through product research questions.

The discover command outputs a structured prompt that guides Claude through
an interview process to define Given/When/Then scenarios for a feature.

Process:
  1. Phase 1: User Personas
     - Who is the primary user?
     - What problem are they solving?
     - Why is this important now?

  2. Phase 2: Happy + Failure Paths
     - What is the ideal successful experience?
     - What variations exist?
     - What could go wrong?
     - How should failures be handled?

  3. Phase 3: Scope Boundaries
     - What is explicitly out of scope?
     - What assumptions are we making?

  4. Synthesis: Claude writes BDD scenarios to the feature file

Why BDD?
  BDD scenarios (Given/When/Then) ensure features are well-defined before
  breaking down into specs. This prevents scope creep and misunderstandings.

Enforcement:
  Features MUST have BDD scenarios before moving from backlog to todo.
  Bugs and tasks skip this validation.

Examples:
  agile.ts discover user-auth
  agile.ts discover dark-mode
`);
        break;
      default:
        console.error(`Unknown command: ${command}`);
        process.exit(1);
    }
    return;
  }

  console.log(`
Agile Workflow Manager (Folder-Based Issues with SPS Specs)

Usage: agile.ts <command> [options]

Issue Commands:
  create <type> <name>    Create a new issue folder (feature, bug, or task)
  discover <name>         Guide BDD scenario creation through product research
  move <name> <stage>     Move an issue to a different stage
  list                    List all issues with spec progress
  work <name>             Analyze an issue and get stage-appropriate guidance
  archive <name>          Archive an issue folder
  delete <name>           Permanently delete an issue folder

Spec Commands:
  spec list <issue>             List specs in an issue
  spec add <issue> <name>       Add a new spec
  spec status <issue> <s> <st>  Update spec status (pending/in-progress/in-review/completed)
  spec delete <issue> <spec>    Delete a spec
  spec suggest <issue>          AI-assisted spec breakdown

Guidance Commands:
  guidance show <issue>         Show technical guidance
  guidance update <issue>       Mark guidance as updated
  guidance validate <issue>     Check if guidance is current

Review Commands:
  review                        Review a spec in in-review status
  fix                           Fix issues from failed review (NEEDS_WORK)

Workflow:
  1-backlog -> 2-todo -> 3-in-progress -> 4-review -> 5-done

BDD Scenarios (Features Only):
  Features require Given/When/Then scenarios before moving to todo.
  Run 'discover' for guided product research and scenario creation.

Issue Structure:
  agile/<stage>/<issue-name>/
    - feature.md (or bug.md/task.md)
    - technical-guidance.md
    - *.spec.md (SPS spec files)

SPS Pattern (Smallest Possible Spec):
  Each spec should be the smallest deliverable unit - roughly the size
  of one "todo" item. All specs must be completed before moving to review.

Examples:
  agile.ts create feature user-auth --title "User Authentication" --owner "Alice"
  agile.ts discover user-auth
  agile.ts spec suggest user-auth
  agile.ts spec status user-auth login-flow in-progress
  agile.ts guidance update user-auth
  agile.ts move user-auth 4-review

Run "agile.ts help <command>" for more information on a command.
`);
}
