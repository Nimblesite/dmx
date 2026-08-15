---
name: upgrade-packages
description: Upgrade all dependencies/packages to their latest versions for the detected language(s). Use when the user says "upgrade packages", "update dependencies", "bump versions", "update packages", or "upgrade deps".
argument-hint: "[--check-only] [--major] [package-name]"
---
<!-- agent-pmo:a72c926 -->

# Upgrade Packages

Upgrade all project dependencies to their latest compatible (or latest major, if `--major`) versions.

## Arguments

- `--check-only` — List outdated packages without upgrading. Stop after Step 2.
- `--major` — Include major version bumps (breaking changes). Without this flag, stay within semver-compatible ranges.
- Any other argument is treated as a specific package name to upgrade (instead of all packages).

## Step 1 — Detect language and package manager

This repo has four manifests. Process each in order:

| Manifest file | Language | Package manager |
|---|---|---|
| `src/dmx/Cargo.toml` | Rust — the `dmx` CLI | cargo |
| `src/editors/vscode/package.json` | Node.js — the VS Code extension | npm (`package-lock.json`) |
| `website/package.json` | Node.js — the site and its Playwright suite | npm (`package-lock.json`) |
| `src/dart_packages/dmx/pubspec.yaml` | Dart — the annotations runtime | pub |

There is **no `Cargo.toml` at the repository root** — the crate is self-contained
at `src/dmx`, so every cargo call below names its manifest. A bare one fails with
`could not find Cargo.toml`.

The example packages under `examples/` are generated samples with path
dependencies on `src/dart_packages/dmx`; they are not upgraded independently.

`.github/dependabot.yml` is the authoritative list — it watches exactly these
four, so check it before you start rather than this table. The repo grows.

## Step 2 — List outdated packages

Run the appropriate command to list what's outdated BEFORE upgrading anything. Show the user what will change.

### Rust
```bash
cargo outdated --manifest-path src/dmx/Cargo.toml   # install: cargo install cargo-outdated
cargo update --manifest-path src/dmx/Cargo.toml --dry-run
```
**Read the docs:** https://doc.rust-lang.org/cargo/commands/cargo-update.html

Note: `tree-sitter-dart` is a **git dependency**, not a crates.io one. `cargo update`
moves it to the newest commit on the tracked branch — check that commit before
accepting it.

### Node.js (npm)
```bash
cd src/editors/vscode && npm outdated
cd website && npm outdated
```
**Read the docs:** https://docs.npmjs.com/cli/v10/commands/npm-update

### Dart
```bash
cd src/dart_packages/dmx && dart pub outdated
```
**Read the docs:** https://dart.dev/tools/pub/cmd/pub-outdated

If `--check-only` was passed, **stop here** and report the outdated list.

## Step 3 — Read the official upgrade docs

**Before running any upgrade command, you MUST fetch and read the official documentation URL listed above for the detected package manager.** Use WebFetch to retrieve the page. This ensures you use the correct flags and understand the behavior. Do not guess at flags or options from memory.

## Step 4 — Upgrade packages

Run the upgrade. If a specific package name was given as an argument, upgrade only that package.

### Rust
```bash
CRATE="--manifest-path src/dmx/Cargo.toml"
cargo update $CRATE                   # semver-compatible updates
# --major flag:
cargo update $CRATE --breaking        # major version bumps (cargo 1.84+)
```

### Node.js (npm)

Once per package — `src/editors/vscode` and `website`:
```bash
cd src/editors/vscode                 # then repeat in website/
npm update                            # semver-compatible (within package.json ranges)
# --major flag:
npx npm-check-updates -u && npm install   # bump package.json to latest majors
```

### Dart
```bash
cd src/dart_packages/dmx
dart pub upgrade                      # semver-compatible
# --major flag:
dart pub upgrade --major-versions     # bump to latest majors
```

## Step 5 — Verify the upgrade

After upgrading, run the project's full gate to confirm nothing broke:

```bash
make ci
```

`make ci` is `fmt CHECK=1` → `lint` → `version-check` → `deslop` → `test`
(fail-fast + coverage threshold) → `dart-package` → `corpus` → `example` →
`example-sqlite` → `example-openapi` → `extension` → `vsix-e2e` → `website` →
`build` — the same gates ci.yml runs, in its order — so it exercises the Rust
crate, the emitted Dart, the packaged extension in a real editor and the browser
playground in one pass. A `tree-sitter` or
`tree-sitter-dart` bump is the highest-risk upgrade here: the front end reads a
CST, so a grammar change can silently alter what `dmx` emits. `make corpus`
regenerating byte-identically is the proof it did not.

If tests fail:
1. Read the failure output carefully
2. Check the changelog / migration guide for the upgraded packages (fetch the release notes URL if available)
3. Fix breaking changes in the code
4. Re-run tests
5. If stuck after 3 attempts on the same failure, report it to the user with the error details and the package that caused it

## Step 6 — Report

Provide a summary:

- Packages upgraded (old version -> new version)
- Packages skipped (and why, e.g., major version bump without `--major` flag)
- Build/test result after upgrade
- Any breaking changes that were fixed
- Any packages that could not be upgraded (with error details)

## Rules

- **Always list outdated packages first** before upgrading anything
- **Always read the official docs** for the package manager before running upgrade commands
- **Always run tests after upgrading** to catch breakage immediately
- **Never remove packages** unless they were explicitly deprecated and replaced
- **Never downgrade packages** unless rolling back a broken upgrade
- **Never modify lockfiles manually** (`Cargo.lock`, `package-lock.json`) — let the package manager regenerate them
- **Never hand-edit regenerated Dart** to make a check pass after an upgrade — fix the template or context builder and regenerate
- **Commit nothing** — leave changes in the working tree for the user to review

## Success criteria

- All outdated packages upgraded to latest compatible (or latest major if `--major`)
- Build passes
- Tests pass
- User has a clear summary of what changed
