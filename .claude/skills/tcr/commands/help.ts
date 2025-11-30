const HELP_TEXT = `
TCR (Test-Commit-Refactor) Skill

Enforces test discipline with explicit test triggers and pre-commit enforcement.

USAGE:
  bun .claude/skills/tcr/tcr.ts <command> [options]

COMMANDS:
  check [step-name] [-v]  Run tests and auto-commit on success (primary command)
  run [files...]          Run tests for specific files
  status [--coverage]     Show current TCR state (step, failures, last result)
  reset                   Reset failure counter to continue past threshold
  coverage [target] [-d]  Run coverage checks (frontend, backend, or both)
  verify-config           Verify coverage thresholds are in sync across all config files
  help                    Show this help message

OPTIONS:
  -v, --verbose   Show full test output (check command, default: condensed)
  -d, --debug     Show detailed coverage output (coverage command)

WORKFLOW:
  1. Write code and tests
  2. Run "tcr check" when ready to test
  3. Tests pass → WIP commit created automatically
  4. Tests fail → fix and run "tcr check" again
  5. Pre-commit hook enforces 100% coverage on all commits

TEST DISCOVERY:
  Frontend: foo.ts → foo.test.ts or foo.spec.ts
  Backend: src-tauri/ changes trigger cargo test
  Backend test files (*_test.rs) auto-excluded from coverage

EXAMPLES:
  bun .claude/skills/tcr/tcr.ts check "Add user auth"
  bun .claude/skills/tcr/tcr.ts coverage --debug
  bun .claude/skills/tcr/tcr.ts status
`;

const COMMAND_HELP: Record<string, string> = {
  check: `
TCR CHECK - Run tests and auto-commit on success

USAGE:
  bun .claude/skills/tcr/tcr.ts check [step-name] [options]

ARGUMENTS:
  step-name   Optional description for the WIP commit (default: "manual check")

OPTIONS:
  --verbose, -v   Show full test output (default: condensed on success)

BEHAVIOR:
  1. Detects changed files via git status
  2. Determines target (frontend/backend/both)
  3. Runs tests with coverage
  4. On success: shows condensed output, creates WIP commit
  5. On failure: shows condensed error details, increments failure counter

OUTPUT:
  Default (quiet mode): "TCR: PASS: Frontend: PASS | Backend: PASS (100%)"
  Verbose mode: Full vitest/cargo output

EXAMPLES:
  bun .claude/skills/tcr/tcr.ts check
  bun .claude/skills/tcr/tcr.ts check "Add user authentication"
  bun .claude/skills/tcr/tcr.ts check --verbose "Debug test issue"
`,

  run: `
TCR RUN - Run tests for specific files

USAGE:
  bun .claude/skills/tcr/tcr.ts run [files...]

ARGUMENTS:
  files    Source files to find tests for (uses convention-based discovery)

EXAMPLES:
  bun .claude/skills/tcr/tcr.ts run src/App.tsx
  bun .claude/skills/tcr/tcr.ts run src/utils/auth.ts src/utils/validation.ts
`,

  status: `
TCR STATUS - Show current TCR state

USAGE:
  bun .claude/skills/tcr/tcr.ts status
  bun .claude/skills/tcr/tcr.ts status --coverage  # Include live coverage metrics

OPTIONS:
  --coverage, -c   Run tests and display current coverage metrics

DISPLAYS:
  - Current step being worked on
  - Failure count for current step
  - Last test result and timestamp
  - Recent errors from .tcr-errors.log (if any)
`,

  reset: `
TCR RESET - Reset failure counter

USAGE:
  bun .claude/skills/tcr/tcr.ts reset

Use this command to continue past the 5-failure threshold when you want
to keep working on the current approach.
`,

  coverage: `
TCR COVERAGE - Run coverage checks

USAGE:
  bun .claude/skills/tcr/tcr.ts coverage [target] [--debug]

ARGUMENTS:
  target   Optional: "frontend", "backend", or omit for both

OPTIONS:
  --debug, -d   Show detailed per-file coverage output (useful for debugging)

EXAMPLES:
  bun .claude/skills/tcr/tcr.ts coverage              # Run both, summary only
  bun .claude/skills/tcr/tcr.ts coverage --debug      # Run both with per-file details
  bun .claude/skills/tcr/tcr.ts coverage frontend     # Frontend only
  bun .claude/skills/tcr/tcr.ts coverage backend -d   # Backend with details
`,

  "verify-config": `
TCR VERIFY-CONFIG - Verify coverage configuration sync

USAGE:
  bun .claude/skills/tcr/tcr.ts verify-config

Checks that coverage thresholds are consistent across all three locations:
  1. TCR config (.claude/skills/tcr/lib/coverage/config.ts)
  2. Vitest config (vitest.config.ts)
  3. Husky pre-commit (.husky/pre-commit)

Exits with code 1 if any mismatches are found.
`,
};

export function handleHelp(args: string[]): void {
  const command = args[0];

  if (command && COMMAND_HELP[command]) {
    console.log(COMMAND_HELP[command]);
  } else {
    console.log(HELP_TEXT);
  }
}
