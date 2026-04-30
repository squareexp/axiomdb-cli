#!/usr/bin/env node
/**
 * PulsarDB CLI — binary shim
 *
 * Resolves the correct pre-built native binary for the current platform
 * from the optionally-installed @pulsardb/cli-<platform> package,
 * then exec-replaces the process with it.
 *
 * This mirrors the pattern used by esbuild, @biomejs/biome, and SWC.
 */

"use strict";

const { spawnSync } = require("child_process");
const path = require("path");
const fs = require("fs");
const os = require("os");

// ── Platform detection ───────────────────────────────────────────────────────

function platformPackageName() {
  const plat = os.platform(); // 'darwin' | 'linux' | 'win32'
  const arch = os.arch();      // 'arm64' | 'x64'

  const supported = {
    "darwin-arm64":  "@pulsardb/cli-darwin-arm64",
    "darwin-x64":    "@pulsardb/cli-darwin-x64",
    "linux-x64":     "@pulsardb/cli-linux-x64",
    "linux-arm64":   "@pulsardb/cli-linux-arm64",
    "win32-x64":     "@pulsardb/cli-win32-x64",
  };

  const key = `${plat}-${arch}`;
  const pkg = supported[key];

  if (!pkg) {
    console.error(
      `pulsardb: Unsupported platform: ${key}\n` +
      `Supported: ${Object.keys(supported).join(", ")}\n` +
      `\nYou can build from source: https://github.com/squareexp/pulsardb`
    );
    process.exit(1);
  }

  return pkg;
}

// ── Binary resolution ────────────────────────────────────────────────────────

function findBinary() {
  const pkg = platformPackageName();
  const binaryName = process.platform === "win32" ? "pulsardb.exe" : "pulsardb";

  // Walk up from the shim to find node_modules/@pulsardb/cli-<platform>
  const candidates = [
    // Installed alongside (standard npm install -g)
    path.join(__dirname, "..", "node_modules", pkg, "bin", binaryName),
    // Hoisted to root node_modules (monorepo / workspaces)
    path.join(__dirname, "..", "..", "..", "node_modules", pkg, "bin", binaryName),
    // Resolved via require.resolve
    (() => {
      try {
        return path.join(
          path.dirname(require.resolve(`${pkg}/package.json`)),
          "bin",
          binaryName
        );
      } catch {
        return null;
      }
    })(),
  ].filter(Boolean);

  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }

  console.error(
    `pulsardb: Could not find the binary for your platform.\n\n` +
    `Expected package: ${pkg}\n\n` +
    `Try re-installing:\n  npm install -g pulsardb\n\n` +
    `Or if you are in a workspace:\n  npm install ${pkg}\n\n` +
    `To build from source:\n  https://github.com/squareexp/pulsardb`
  );
  process.exit(1);
}

// ── Run ──────────────────────────────────────────────────────────────────────

const binary = findBinary();

const result = spawnSync(binary, process.argv.slice(2), {
  stdio: "inherit",
  // Pass through environment so config paths, $HOME, etc. are correct
  env: process.env,
});

if (result.error) {
  console.error(`pulsardb: Failed to start binary: ${result.error.message}`);
  process.exit(1);
}

process.exit(result.status ?? 0);
