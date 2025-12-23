import { describe, expect, it, beforeEach, afterEach } from "bun:test";
import { existsSync, mkdirSync, rmSync, writeFileSync, statSync } from "fs";
import { tmpdir } from "os";
import { join } from "path";

// Import the functions we want to test
import {
  detectWorktreeContext,
  parseArgs,
  getRepoBranch,
  checkMainRepoClean,
  getAgileDiff,
  type WorktreeInfo,
} from "./sync-agile";

describe("sync-agile", () => {
  describe("parseArgs", () => {
    it("returns default values for empty args", () => {
      const flags = parseArgs([]);
      expect(flags.dryRun).toBe(false);
      expect(flags.force).toBe(false);
      expect(flags.noCommit).toBe(false);
      expect(flags.help).toBe(false);
      expect(flags.message).toBe("Sync agile/ from worktree");
    });

    it("recognizes --dry-run flag", () => {
      const flags = parseArgs(["--dry-run"]);
      expect(flags.dryRun).toBe(true);
    });

    it("recognizes --force flag", () => {
      const flags = parseArgs(["--force"]);
      expect(flags.force).toBe(true);
    });

    it("recognizes -f as shorthand for --force", () => {
      const flags = parseArgs(["-f"]);
      expect(flags.force).toBe(true);
    });

    it("recognizes --no-commit flag", () => {
      const flags = parseArgs(["--no-commit"]);
      expect(flags.noCommit).toBe(true);
    });

    it("recognizes --help flag", () => {
      const flags = parseArgs(["--help"]);
      expect(flags.help).toBe(true);
    });

    it("recognizes -h as shorthand for --help", () => {
      const flags = parseArgs(["-h"]);
      expect(flags.help).toBe(true);
    });

    it("parses -m message argument", () => {
      const flags = parseArgs(["-m", "Custom message"]);
      expect(flags.message).toBe("Custom message");
    });

    it("parses --message argument", () => {
      const flags = parseArgs(["--message", "Another message"]);
      expect(flags.message).toBe("Another message");
    });

    it("handles multiple flags together", () => {
      const flags = parseArgs(["--dry-run", "--force", "-m", "Test"]);
      expect(flags.dryRun).toBe(true);
      expect(flags.force).toBe(true);
      expect(flags.message).toBe("Test");
    });
  });

  describe("detectWorktreeContext", () => {
    it("returns null when run from main repository", async () => {
      // When run from the main heycat repo, should return null
      // (unless the test itself is running from a worktree)
      const result = await detectWorktreeContext();

      // We're testing the detection logic - in main repo it should be null
      // If .git is a directory, we're in main repo
      const gitPath = join(process.cwd(), ".git");
      if (existsSync(gitPath)) {
        const stat = statSync(gitPath);
        if (stat.isDirectory()) {
          expect(result).toBeNull();
        } else {
          // We're in a worktree - result should have required fields
          expect(result).not.toBeNull();
          expect(result).toHaveProperty("identifier");
          expect(result).toHaveProperty("mainRepoPath");
          expect(result).toHaveProperty("worktreePath");
          expect(result).toHaveProperty("gitdirPath");
        }
      }
    });

    it("returns WorktreeInfo with required fields when in worktree", async () => {
      const result = await detectWorktreeContext();
      if (result !== null) {
        expect(typeof result.identifier).toBe("string");
        expect(typeof result.mainRepoPath).toBe("string");
        expect(typeof result.worktreePath).toBe("string");
        expect(typeof result.gitdirPath).toBe("string");
        expect(result.identifier.length).toBeGreaterThan(0);
      }
    });
  });

  describe("getRepoBranch", () => {
    it("returns the current branch name for a valid repo", async () => {
      // Test with the current working directory (should be a git repo)
      const branch = await getRepoBranch(process.cwd());
      expect(typeof branch).toBe("string");
      expect(branch.length).toBeGreaterThan(0);
    });
  });

  describe("checkMainRepoClean", () => {
    it("returns a boolean for the current repo", async () => {
      const isClean = await checkMainRepoClean(process.cwd());
      expect(typeof isClean).toBe("boolean");
    });
  });

  describe("getAgileDiff", () => {
    const testDir = join(tmpdir(), `heycat-sync-test-${Date.now()}`);
    const dir1 = join(testDir, "worktree", "agile");
    const dir2 = join(testDir, "main", "agile");

    beforeEach(() => {
      mkdirSync(dir1, { recursive: true });
      mkdirSync(dir2, { recursive: true });
    });

    afterEach(() => {
      if (existsSync(testDir)) {
        rmSync(testDir, { recursive: true, force: true });
      }
    });

    it("returns empty string when directories are identical", async () => {
      // Create identical files in both
      writeFileSync(join(dir1, "test.md"), "same content");
      writeFileSync(join(dir2, "test.md"), "same content");

      const diff = await getAgileDiff(join(testDir, "worktree"), join(testDir, "main"));
      expect(diff).toBe("");
    });

    it("detects files only in one directory", async () => {
      // Create file only in worktree
      writeFileSync(join(dir1, "new-file.md"), "content");

      const diff = await getAgileDiff(join(testDir, "worktree"), join(testDir, "main"));
      expect(diff).toContain("new-file.md");
    });

    it("detects differing files", async () => {
      // Create files with different content
      writeFileSync(join(dir1, "test.md"), "content A");
      writeFileSync(join(dir2, "test.md"), "content B");

      const diff = await getAgileDiff(join(testDir, "worktree"), join(testDir, "main"));
      expect(diff).toContain("differ");
    });
  });

  describe("WorktreeInfo interface", () => {
    it("can create a valid WorktreeInfo object", () => {
      const info: WorktreeInfo = {
        identifier: "test-worktree",
        mainRepoPath: "/path/to/main",
        worktreePath: "/path/to/worktree",
        gitdirPath: "/path/to/main/.git/worktrees/test-worktree",
      };

      expect(info.identifier).toBe("test-worktree");
      expect(info.mainRepoPath).toBe("/path/to/main");
      expect(info.worktreePath).toBe("/path/to/worktree");
      expect(info.gitdirPath).toBe("/path/to/main/.git/worktrees/test-worktree");
    });
  });

  describe("gitdir path parsing", () => {
    // Test the logic used to parse worktree gitdir paths
    it("extracts identifier from gitdir path", () => {
      const gitdirPath = "/path/to/repo/.git/worktrees/heycat-feature-branch";
      const identifier = gitdirPath.split("/").pop() || "";
      expect(identifier).toBe("heycat-feature-branch");
    });

    it("navigates from gitdir to main repo path", () => {
      const gitdirPath = "/path/to/repo/.git/worktrees/heycat-feature-branch";

      // gitdir: .git/worktrees/<name>
      // Parent of parent of gitdir is .git
      // Parent of .git is repo root
      const parts = gitdirPath.split("/");
      parts.pop(); // remove identifier
      parts.pop(); // remove "worktrees"
      const gitDir = parts.join("/"); // .git
      parts.pop(); // remove ".git"
      const mainRepoPath = parts.join("/");

      expect(gitDir).toBe("/path/to/repo/.git");
      expect(mainRepoPath).toBe("/path/to/repo");
    });
  });
});
