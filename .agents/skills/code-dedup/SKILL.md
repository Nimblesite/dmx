---
name: code-dedup
description: Searches for duplicate code, duplicate tests, and dead code, then safely merges or removes them. Use when the user says "deduplicate", "find duplicates", "remove dead code", "DRY up", or "code dedup". Requires test coverage — refuses to touch untested code.
---
<!-- agent-pmo:a72c926 -->

# Code Dedup

Find duplicate code, duplicate tests, and dead code across the repo. Merge duplicates and delete dead code — but only when test coverage proves the change is safe.

## Prerequisites — hard gate

Stop and report if any of these fail:

1. **Tests green.** Run `make test` — it is fail-fast AND enforces the coverage threshold from `coverage-thresholds.json` (REPO-STANDARDS-SPEC [TEST-RULES]). Non-zero exit = stop. Never dedup a broken or under-covered codebase. Note the current coverage % — it is the floor and must not drop.
2. **Static typing.** Rust and Dart are typed by default; `make lint` (clippy, warnings denied) and `dart analyze --fatal-infos` are the checkers. Both must be clean before you start.
3. **deslop reachable** (see below), or the CLI fallback declared.

## ⛔️ Never dedup generator output

`example/lib/**` and the region below the divider in `tests/golden/*.dart` are
**dmx OUTPUT**, not source. They are *supposed* to look alike — sibling data
classes emitted from the same template are structurally identical by design, and
they are the fixtures the golden tests assert on byte-for-byte. deslop's current
top clusters live there.

- **Never merge, extract, or edit a cluster whose occurrences are all in generated Dart.** Mark it `(generator output — by design)` in the report and move on.
- Real duplication in the emitted shape is fixed in `templates/*.mustache` or `src/context.rs`, then regenerated with `make golden` / `make example` — never by hand-editing the output.
- `tests/golden/*.dart` above the divider is hand-written input; treat it as fixture data, not logic to DRY up.

## Required tooling — deslop

This skill is driven by **deslop** (docs: https://deslop.live/docs/for-ai/). It is the duplicate scanner — do not substitute grep or eyeballing. Both of this repo's languages, **`rust`** and **`dart`**, are supported.

**The MCP server is PREFERRED.** Use it when available. Key tools:
- `mcp__deslop__top-offenders` — worst clusters first (primary input to Step 3).
- `mcp__deslop__report-query` — AND-filter by `bucket`, `path_contains`, `language`, `min_score`, `min_size`; paginate with `offset`/`limit`.
- `mcp__deslop__cluster-by-id` — full record for one cluster before merging.
- `mcp__deslop__report-for-file` / `report-for-range` — narrow to a file/range during merge planning.
- `mcp__deslop__find-similar` — call BEFORE writing any replacement. Reuse if `signals.fused ≥ 0.85` or bucket `identical`/`nearly_identical`; write new if `fused < 0.6` or empty; bias to reuse in between.
- `mcp__deslop__rescan` — refresh the index; call after each merge/deletion to confirm the cluster is gone.

**If the MCP server is unavailable, run the CLI instead** (same `.deslop.toml`, same report):
- `make deslop` — full workspace scan; reads `.deslop.toml`, writes `.deslop/deslop-report.json`, exits 3 when duplication exceeds the stored threshold. **This is exactly the CI gate** — the threshold lives in `.deslop.toml`, never a CLI/YAML number ([CI-DESLOP]).
- `deslop . --no-fail-over --output .deslop/deslop-report` — same scan, but never fails on breach; local inspection only.

Parse **only `.deslop/deslop-report.json`** (the `.txt`/`.html` outputs are for humans). Decision fields: `metrics.duplication_percent`, `metrics.threshold.breached`, `clusters[].weight` (sorted desc), `clusters[].signals.fused`, `clusters[].bucket`. Re-run `make deslop` after each change in place of `rescan`.

**Buckets** (act in this order): `identical` > `nearly_identical` > `loosely_similar` > `same_behavior`. `identical` is pure copy-paste and safest to merge. `same_behavior` needs human judgement — only on explicit request.

**Rule:** every cluster you act on must be cited in the final report by its cluster ID, bucket, and score/`fused`. No anonymous "found some duplicates".

## Steps

```
Dedup Progress:
- [ ] Step 1: Prerequisites passed (tests green, coverage noted, typed, deslop MCP or CLI confirmed)
- [ ] Step 2: Dead code scan complete
- [ ] Step 3: Duplicate code scan complete via deslop
- [ ] Step 4: Duplicate test scan complete via deslop (filtered to test paths)
- [ ] Step 5: Changes applied — each merge preceded by find-similar, followed by rescan/re-run
- [ ] Step 6: Verification passed (tests green, coverage stable, deslop confirms targeted clusters gone)
```

### Step 1 — Inventory coverage
Confirm the green baseline from the prerequisites. Only files WITH coverage are candidates — leave untested files alone.

### Step 2 — Scan for dead code
Find code never called, imported, or referenced. Use the language's own signal first: `cargo clippy --all-targets -- -D warnings` (the crate denies `dead_code` and `unused_imports` outright) for Rust, `dart analyze --fatal-infos` for `src/dart_packages/dmx`. For each candidate, grep the whole codebase (tests, `templates/`, scripts, configs) — only dead if truly zero references. Watch for Rust items reached only through the `lspkit` engine contract or the Mustache templates: a name that appears in `templates/*.mustache` is live. List with file:line. Do NOT delete yet.

### Step 3 — Scan for duplicate code (deslop)
1. MCP: `top-offenders`, then `report-query { bucket: "identical" }`, then `nearly_identical`, then `loosely_similar`. CLI: run `make deslop` and read clusters from `.deslop/deslop-report.json`, worst `weight` first.
2. Drop every cluster whose occurrences are all under `example/lib`, `tests/golden`, or `target/` — that is generator output (see above).
3. For each remaining cluster you intend to act on, fetch the full record (`cluster-by-id` / the report entry) and read every occurrence at its byte ranges. deslop measures structure, not semantics — if occurrences differ on a subtle condition or default, leave them and note "false positive".
4. Record `{ cluster_id, bucket, score, occurrences[], decision, rationale }`. Do NOT merge yet.

### Step 4 — Scan for duplicate tests (deslop)
Same as Step 3, filtered to test paths: `report-query { path_contains: "tests" }`, or filter `.deslop/deslop-report.json` clusters by occurrence path. Keep the more thorough test — the black-box end-to-end suite wins over a narrower one. Flag shared test fixtures/helpers as merge candidates rather than deletions.

### Step 5 — Apply changes (one at a time)
Cycle per change: **change → `make test` → check coverage → continue or revert.**

**5a. Dead code:** delete, then `make test`. Non-zero exit = revert.

**5b. Merge duplicate code:** pick ONE cluster (worst `identical` first). Call `find-similar` before writing the replacement — reuse an existing canonical if it returns one. Extract shared logic, update call sites, `make test`. Tests fail = revert (subtle semantic difference). Coverage drops = add tests first. Then `rescan` (or re-run `make deslop`) to confirm the cluster is gone and no new one appeared.

**5c. Duplicate tests:** delete the redundant test (keep the thorough one), `make test`. Coverage drop = revert (it covered something the other didn't). Confirm the cluster is gone.

### Step 6 — Final verification
1. `make lint` — clippy passes with warnings denied.
2. `make test` — green AND coverage ≥ the Step 1 floor.
3. `make corpus example` — the emitted Dart still regenerates byte-identically and analyzes clean.
4. Final `rescan` / `make deslop` — every cluster you acted on resolved, top-offender list shorter than at Step 3 start. If duplication fell, ratchet `max_duplication_percent` in `.deslop.toml` DOWN in the same change.
5. Report: every cluster ID acted on (bucket, score, occurrences merged/deleted), every cluster skipped as generator output, the new top-offenders list, and final coverage vs baseline.

## Rules

- **deslop is the scanner.** MCP preferred, CLI acceptable. Cite every cluster by ID, bucket, and score. Unreachable MCP → use the CLI; never silently fall back to grep.
- **Generated Dart is off limits.** Fix the template, not the output.
- **No coverage = do not touch.** You cannot safely dedup what you cannot verify.
- **Coverage must not drop.** The Step 1 floor is sacred — revert anything that lowers it.
- **One change at a time.** Never batch dedup changes before testing.
- **When in doubt, leave it.** False dedup is worse than duplication.
- **Preserve public API.** Internal refactoring only — no signature/export changes external code depends on.
- **Trivial duplication is fine.** Only dedup substantial shared logic (>10 lines) or 3+ copies.
