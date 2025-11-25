const HELP_TEXT = `
TCR (Test-Commit-Refactor) Skill

Automates the test-commit-refactor workflow using Claude Code hooks.

USAGE:
  bun .claude/skills/tcr/tcr.ts <command> [options]

COMMANDS:
  run [files...]   Run tests for specific files
  status           Show current TCR state (step, failures, last result)
  reset            Reset failure counter to continue past threshold
  help             Show this help message

WORKFLOW:
  1. Mark a todo as "completed" → tests run automatically
  2. Tests pass → WIP commit created
  3. Tests fail → failure counter increments
  4. 5 failures → prompted to reconsider approach
  5. Git commits blocked until tests pass

TEST DISCOVERY:
  Convention-based: foo.ts → foo.test.ts or foo.spec.ts
  Backend: src-tauri/ changes trigger cargo test

EXAMPLES:
  bun .claude/skills/tcr/tcr.ts run src/App.tsx
  bun .claude/skills/tcr/tcr.ts status
  bun .claude/skills/tcr/tcr.ts reset
`;

const COMMAND_HELP: Record<string, string> = {
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

DISPLAYS:
  - Current step being worked on
  - Failure count for current step
  - Last test result and timestamp
`,

  reset: `
TCR RESET - Reset failure counter

USAGE:
  bun .claude/skills/tcr/tcr.ts reset

Use this command to continue past the 5-failure threshold when you want
to keep working on the current approach.
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
