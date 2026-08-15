// Builds the production string pipeline for the browser [playground.wasm].
import { existsSync, mkdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const repository = resolve(dirname(fileURLToPath(import.meta.url)), "..");
// The crate is self-contained at src/dmx and the repository root carries no
// Cargo.toml, so every command here runs from the crate. `rustc` and
// `wasm-bindgen` do not read the working directory, so one cwd serves all.
const crate = join(repository, "src", "dmx");

function fail(message) {
  process.stderr.write(`error: ${message}\n`);
  process.exit(1);
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: crate,
    encoding: "utf8",
    ...options,
  });
  if (result.error !== undefined) {
    fail(`${command} could not start: ${result.error.message}`);
  }
  if (result.status !== 0) {
    if (result.stdout) process.stdout.write(result.stdout);
    if (result.stderr) process.stderr.write(result.stderr);
    fail(`${command} exited with status ${String(result.status)}`);
  }
  return result.stdout;
}

const manifestPath = join(crate, "Cargo.toml");
const metadata = JSON.parse(
  run("cargo", ["metadata", "--format-version", "1", "--locked", "--manifest-path", manifestPath]),
);
const languagePackage = metadata.packages.find((candidate) => candidate.name === "tree-sitter-language");
const bindgenPackage = metadata.packages.find((candidate) => candidate.name === "wasm-bindgen");
if (languagePackage === undefined || bindgenPackage === undefined) {
  fail("the Tree-sitter headers or wasm-bindgen package are missing from Cargo metadata");
}

const portableHeaders = join(dirname(languagePackage.manifest_path), "wasm", "include");
if (!existsSync(join(portableHeaders, "stdlib.h"))) {
  fail("Tree-sitter's portable WASM headers were not found");
}

const bindgenVersion = run("wasm-bindgen", ["--version"]).trim().split(" ").at(-1);
if (bindgenVersion !== bindgenPackage.version) {
  fail(`wasm-bindgen ${bindgenPackage.version} is required; run 'make setup'`);
}

// `llvm-ar` as shipped by the Rust toolchain's llvm-tools component, which is
// version-matched to the compiler and is present wherever that component is —
// including CI, which already installs it for coverage. A distribution's own
// `llvm` package is the alternative, and a worse one: it is named differently
// on every platform, and its absence surfaced here as a cc-rs failure deep
// inside a cargo build rather than as anything anybody could act on.
function toolchainArchiver() {
  const host = run("rustc", ["-vV"])
    .split("\n")
    .find((line) => line.startsWith("host: "))
    ?.slice("host: ".length)
    .trim();
  if (host === undefined) {
    return undefined;
  }
  const archiver = join(
    run("rustc", ["--print", "sysroot"]).trim(),
    "lib",
    "rustlib",
    host,
    "bin",
    "llvm-ar",
  );
  return existsSync(archiver) ? archiver : undefined;
}

const homebrewPrefixes = ["/opt/homebrew/opt/llvm", "/usr/local/opt/llvm"];
const homebrewPrefix = homebrewPrefixes.find((prefix) => existsSync(join(prefix, "bin", "clang")));
const homebrewClang = homebrewPrefix === undefined ? undefined : join(homebrewPrefix, "bin", "clang");
const wasmCompiler = process.env.WASM_CC
  ?? homebrewClang
  ?? "clang";
const targetFlags = [`-I${portableHeaders}`, process.env.CFLAGS_wasm32_unknown_unknown ?? ""]
  .filter((value) => value.length > 0)
  .join(" ");
const buildEnvironment = {
  ...process.env,
  AR_wasm32_unknown_unknown: process.env.WASM_AR
    ?? (homebrewPrefix === undefined ? undefined : join(homebrewPrefix, "bin", "llvm-ar"))
    ?? toolchainArchiver()
    ?? "llvm-ar",
  CC_wasm32_unknown_unknown: wasmCompiler,
  CFLAGS_wasm32_unknown_unknown: targetFlags,
};

run(
  "cargo",
  [
    "build",
    "--locked",
    "--lib",
    "--release",
    "--target",
    "wasm32-unknown-unknown",
    "--manifest-path",
    manifestPath,
  ],
  { env: buildEnvironment, stdio: "inherit" },
);

const targetDir = join(crate, "target");
const wasm = join(targetDir, "wasm32-unknown-unknown", "release", "dmx.wasm");
const webOutput = join(repository, "website", "pkg");
const nodeOutput = join(targetDir, "wasm-node");
mkdirSync(webOutput, { recursive: true });
mkdirSync(nodeOutput, { recursive: true });
run("wasm-bindgen", ["--target", "web", "--out-dir", webOutput, "--out-name", "dmx_wasm", wasm], {
  stdio: "inherit",
});
run("wasm-bindgen", ["--target", "nodejs", "--out-dir", nodeOutput, "--out-name", "dmx_wasm", wasm], {
  stdio: "inherit",
});
