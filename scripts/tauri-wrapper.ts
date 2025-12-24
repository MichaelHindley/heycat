#!/usr/bin/env bun
/**
 * Tauri CLI wrapper that handles worktree-specific port configuration.
 *
 * This wrapper:
 * 1. Detects if running in a worktree
 * 2. Calculates the appropriate dev server port
 * 3. Injects the port configuration when running `tauri dev`
 * 4. Passes all other commands through to Tauri CLI unchanged
 *
 * Usage: bun scripts/tauri-wrapper.ts [tauri-args...]
 *
 * Examples:
 *   bun scripts/tauri-wrapper.ts dev        # Runs dev with correct port
 *   bun scripts/tauri-wrapper.ts build      # Passes through to tauri build
 *   bun scripts/tauri-wrapper.ts --help     # Passes through to tauri --help
 */

import { existsSync, readFileSync } from "fs";
import { homedir } from "os";
import { resolve } from "path";
import { getDevPort, getWorktreeIdentifier } from "./dev-port";

/**
 * Get the heycat application support directory path.
 */
function getAppSupportDir(): string {
  const home = homedir();
  if (process.platform === "darwin") {
    return resolve(home, "Library/Application Support/com.heycat.app");
  }
  return resolve(home, ".local/share/com.heycat.app");
}

/**
 * Get the recording hotkey from the settings file.
 * Returns null if not configured or file doesn't exist.
 */
function getRecordingHotkey(worktreeIdentifier: string | null): string | null {
  const settingsFileName = worktreeIdentifier
    ? `settings-${worktreeIdentifier}.json`
    : "settings.json";
  const settingsPath = resolve(getAppSupportDir(), settingsFileName);

  if (!existsSync(settingsPath)) {
    return null;
  }

  try {
    const content = readFileSync(settingsPath, "utf-8");
    const settings = JSON.parse(content);
    return settings["hotkey.recordingShortcut"] || null;
  } catch {
    return null;
  }
}

const args = process.argv.slice(2);
const identifier = getWorktreeIdentifier();
const port = getDevPort(identifier);

// Check if this is a `dev` command (first non-flag arg is "dev")
const isDevCommand = args.some((arg, index) => {
  // Skip flags
  if (arg.startsWith("-")) return false;
  // First positional arg
  return arg === "dev";
});

// Build the tauri command
const tauriArgs = [...args];

if (isDevCommand) {
  // Inject port configuration for dev command
  const configOverride = JSON.stringify({
    build: {
      devUrl: `http://localhost:${port}`,
    },
  });

  // Add --config flag if not already present
  const hasConfig = args.some((arg) => arg === "--config" || arg.startsWith("--config="));
  if (!hasConfig) {
    tauriArgs.push("--config", configOverride);
  }

  // Log port and hotkey info for visibility
  if (identifier) {
    console.log(`[tauri-wrapper] Worktree: ${identifier}`);
  }
  console.log(`[tauri-wrapper] Dev server port: ${port}`);
  const hotkey = getRecordingHotkey(identifier);
  console.log(`[tauri-wrapper] Recording hotkey: ${hotkey || "(not configured)"}`);
}

// Set environment variables for Vite and Rust
const env = {
  ...process.env,
  VITE_DEV_PORT: String(port),
  HEYCAT_WORKTREE_ID: identifier || "", // Empty string = main repo
};

// Run tauri CLI via bunx to use local installation
const proc = Bun.spawn(["bunx", "tauri", ...tauriArgs], {
  env,
  stdout: "inherit",
  stderr: "inherit",
  stdin: "inherit",
});

// Wait for completion and exit with same code
const exitCode = await proc.exited;
process.exit(exitCode);
