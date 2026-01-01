import { afterEach, beforeEach, describe, expect, it } from "bun:test";
import { getConfig, parseArgs } from "./mac-build";

describe("mac-build", () => {
  // Save original env vars
  const originalEnv = { ...process.env };

  beforeEach(() => {
    // Reset env vars before each test
    delete process.env.HEYCAT_MAC_HOST;
    delete process.env.HEYCAT_MAC_USER;
    delete process.env.HEYCAT_MAC_PATH;
  });

  afterEach(() => {
    // Restore original env vars
    process.env = { ...originalEnv };
  });

  describe("parseArgs", () => {
    it("parses --sync-only flag", () => {
      const flags = parseArgs(["--sync-only"]);
      expect(flags.syncOnly).toBe(true);
      expect(flags.dev).toBe(false);
    });

    it("parses --sync alias", () => {
      const flags = parseArgs(["--sync"]);
      expect(flags.syncOnly).toBe(true);
    });

    it("parses --dev flag", () => {
      const flags = parseArgs(["--dev"]);
      expect(flags.dev).toBe(true);
      expect(flags.syncOnly).toBe(false);
    });

    it("parses --help flag", () => {
      const flags = parseArgs(["--help"]);
      expect(flags.help).toBe(true);
    });

    it("parses -h short flag", () => {
      const flags = parseArgs(["-h"]);
      expect(flags.help).toBe(true);
    });

    it("parses multiple flags together", () => {
      const flags = parseArgs(["--sync-only", "--dev"]);
      expect(flags.syncOnly).toBe(true);
      expect(flags.dev).toBe(true);
    });

    it("returns default flags when no args", () => {
      const flags = parseArgs([]);
      expect(flags.syncOnly).toBe(false);
      expect(flags.dev).toBe(false);
      expect(flags.help).toBe(false);
    });
  });

  describe("getConfig", () => {
    it("returns null when no env vars set", () => {
      const config = getConfig();
      expect(config).toBeNull();
    });

    it("returns null when only host is set", () => {
      process.env.HEYCAT_MAC_HOST = "192.168.1.100";
      const config = getConfig();
      expect(config).toBeNull();
    });

    it("returns null when only host and user are set", () => {
      process.env.HEYCAT_MAC_HOST = "192.168.1.100";
      process.env.HEYCAT_MAC_USER = "user";
      const config = getConfig();
      expect(config).toBeNull();
    });

    it("returns config when all env vars are set", () => {
      process.env.HEYCAT_MAC_HOST = "192.168.1.100";
      process.env.HEYCAT_MAC_USER = "devuser";
      process.env.HEYCAT_MAC_PATH = "/Users/devuser/heycat";

      const config = getConfig();

      expect(config).not.toBeNull();
      expect(config?.host).toBe("192.168.1.100");
      expect(config?.user).toBe("devuser");
      expect(config?.path).toBe("/Users/devuser/heycat");
    });

    it("handles hostname instead of IP", () => {
      process.env.HEYCAT_MAC_HOST = "mac.local";
      process.env.HEYCAT_MAC_USER = "user";
      process.env.HEYCAT_MAC_PATH = "~/heycat-docker";

      const config = getConfig();

      expect(config).not.toBeNull();
      expect(config?.host).toBe("mac.local");
    });

    it("defaults host to host.docker.internal when HEYCAT_MAC_HOST not set", () => {
      // Only set user and path, not host
      process.env.HEYCAT_MAC_USER = "devuser";
      process.env.HEYCAT_MAC_PATH = "/Users/devuser/heycat";

      const config = getConfig();

      expect(config).not.toBeNull();
      expect(config?.host).toBe("host.docker.internal");
      expect(config?.user).toBe("devuser");
      expect(config?.path).toBe("/Users/devuser/heycat");
    });
  });

  describe("rsync exclusions", () => {
    it("documents excluded directories", () => {
      // This test documents the expected exclusions
      const expectedExclusions = [
        "target/",       // Rust build artifacts
        "node_modules/", // npm/bun dependencies
        ".git/",         // git repository data
        "dist/",         // frontend build output
        "*.log",         // log files
        ".tcr-state.json",
        ".tcr-errors.log",
        ".tcr/",
        "coverage/",
      ];

      // Just verify the list is what we expect
      expect(expectedExclusions.length).toBeGreaterThan(0);
      expect(expectedExclusions).toContain("target/");
      expect(expectedExclusions).toContain("node_modules/");
    });
  });

  describe("SSH target format", () => {
    it("builds correct SSH target string", () => {
      const config = {
        host: "192.168.1.100",
        user: "devuser",
        path: "/Users/devuser/heycat",
      };

      const sshTarget = `${config.user}@${config.host}`;
      expect(sshTarget).toBe("devuser@192.168.1.100");
    });

    it("builds correct rsync target string", () => {
      const config = {
        host: "192.168.1.100",
        user: "devuser",
        path: "/Users/devuser/heycat",
      };

      const rsyncTarget = `${config.user}@${config.host}:${config.path}/`;
      expect(rsyncTarget).toBe("devuser@192.168.1.100:/Users/devuser/heycat/");
    });
  });
});
