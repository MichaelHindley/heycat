import { STAGES, STAGE_NAMES } from "../lib/types";

export function handleHelp(args: string[]): void {
  const [command] = args;

  if (command) {
    switch (command) {
      case "create":
        console.log(`
Usage: agile.ts create <type> <name> [options]

Create a new issue from a template.

Arguments:
  type     Issue type: feature, bug, or task
  name     Kebab-case name for the issue (e.g., user-authentication)

Options:
  --title, -t    Human-readable title (defaults to name in Title Case)
  --owner, -o    Issue owner/assignee name
  --stage, -s    Initial stage (default: 1-backlog)

Examples:
  agile.ts create feature user-auth --title "User Authentication" --owner "Alice"
  agile.ts create bug fix-login --stage 2-todo --owner "Bob"
`);
        break;
      case "move":
        console.log(`
Usage: agile.ts move <name> <stage>

Move an issue to a different workflow stage.

Arguments:
  name     Issue name (with or without .md extension)
  stage    Target stage: ${STAGES.join(", ")}

Workflow (only sequential transitions allowed):
  1-backlog -> 2-todo -> 3-in-progress -> 4-review -> 5-done

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

Archive an issue (move to agile/archive/ with timestamp).

Arguments:
  name     Issue name to archive

Examples:
  agile.ts archive completed-feature
`);
        break;
      case "delete":
        console.log(`
Usage: agile.ts delete <name>

Permanently delete an issue.

Arguments:
  name     Issue name to delete

Examples:
  agile.ts delete old-task
`);
        break;
      default:
        console.error(`Unknown command: ${command}`);
        process.exit(1);
    }
    return;
  }

  console.log(`
Agile Workflow Manager

Usage: agile.ts <command> [options]

Commands:
  create <type> <name>    Create a new issue (feature, bug, or task)
  move <name> <stage>     Move an issue to a different stage
  list                    List all issues
  archive <name>          Archive an issue
  delete <name>           Permanently delete an issue
  help [command]          Show help for a command

Workflow Stages:
  1-backlog -> 2-todo -> 3-in-progress -> 4-review -> 5-done

Only sequential transitions are allowed (forward or back by one stage).

Examples:
  agile.ts create feature user-auth --title "User Authentication" --owner "Alice"
  agile.ts move user-auth 2-todo
  agile.ts list --stage 3-in-progress
  agile.ts archive completed-feature

Run "agile.ts help <command>" for more information on a command.
`);
}
