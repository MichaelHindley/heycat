import { describe, expect, it } from "bun:test";
import { parseArgs } from "./close-worktree";

describe("close-worktree", () => {
  describe("parseArgs", () => {
    it("defaults to no flags when no arguments", () => {
      const flags = parseArgs([]);
      expect(flags.force).toBe(false);
      expect(flags.help).toBe(false);
    });

    it("parses --force flag", () => {
      const flags = parseArgs(["--force"]);
      expect(flags.force).toBe(true);
    });

    it("parses -f shorthand for --force", () => {
      const flags = parseArgs(["-f"]);
      expect(flags.force).toBe(true);
    });

    it("parses --help flag", () => {
      const flags = parseArgs(["--help"]);
      expect(flags.help).toBe(true);
    });

    it("parses -h shorthand for --help", () => {
      const flags = parseArgs(["-h"]);
      expect(flags.help).toBe(true);
    });

    it("combines multiple flags", () => {
      const flags = parseArgs(["--force", "--help"]);
      expect(flags.force).toBe(true);
      expect(flags.help).toBe(true);
    });

    it("ignores unknown arguments", () => {
      const flags = parseArgs(["--unknown", "some-value"]);
      expect(flags.force).toBe(false);
      expect(flags.help).toBe(false);
    });
  });
});
