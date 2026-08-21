# agent-pmo:a72c926
.DEFAULT_GOAL := help
# `example` is also a directory name, and macOS filesystems are
# case-insensitive — declaring the case variants phony keeps `make EXAMPLE`
# from matching the directory and reporting "nothing to be done".
.PHONY: help build test lint fmt clean ci setup dart-package example EXAMPLE run-example example-openapi example-openapi-live dev watch golden corpus extension vsix vsix-universal vsix-e2e rebuild rebuild-install-vsix wasm wasm-test website version version-check release-check dart-package-version deslop

# All code lives under src/. The Rust crate is self-contained at src/dmx — it
# carries its own Cargo.toml, Cargo.lock, tests/ and templates/ — so cargo is
# always invoked through its manifest and never depends on the shell's cwd.
# `--manifest-path` is a SUBCOMMAND flag, never a global one: `cargo
# --manifest-path … fmt` is an error, so $(CRATE) goes after the subcommand.
CRATE_DIR := src/dmx
CRATE := --manifest-path $(CRATE_DIR)/Cargo.toml
TARGET_DIR := $(CRATE_DIR)/target

EXAMPLE_DIR := examples/storefront
CORPUS_DIR := $(TARGET_DIR)/corpus
DMX_PACKAGE_DIR := src/dart_packages/dmx
GOLDEN_DIR := $(CRATE_DIR)/tests/golden
TD_GOLDEN_DIR := $(CRATE_DIR)/tests/typediagram/golden

# Every Dart directory a human writes. Deliberately enumerated rather than
# globbed: what is NOT here is dmx output, and formatting output rewrites the
# bytes the golden and example gates assert on.
DART_HAND_WRITTEN := \
  src/dart_packages/dmx \
  examples/dmx_sqlite_example/tool \
  examples/dmx_sqlite_example/test \
  examples/dmx_openapi_example/tool \
  examples/dmx_openapi_example/test \
  examples/dmx_openapi_example/test_live

# `dart format` formats to the language version of the surrounding package, and
# looks for a package config to find it — so with no `.dart_tool` it falls back
# to the latest version and formats in a DIFFERENT style. `fmt` runs before any
# `dart pub get`, which made the gate depend on whether a developer happened to
# have resolved the packages: 0 files changed locally, 26 on a clean CI runner.
# Stated explicitly, it is the same answer everywhere. This is the value all
# three pubspecs declare (`sdk: ^3.0.0`) — bump it WITH them, in a commit that
# also reformats, exactly like the SDK pin in ci.yml.
DART_LANGUAGE_VERSION := 3.0
WEBSITE_DIR := website
WASM_BINDGEN_VERSION := 0.2.114

# ---------------------------------------------------------------------------
# OS Detection ([MAKE-CROSS-PLATFORM])
# ---------------------------------------------------------------------------
ifeq ($(OS),Windows_NT)
  SHELL := powershell.exe
  .SHELLFLAGS := -NoProfile -Command
  RM = Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
  MKDIR = New-Item -ItemType Directory -Force
  HOME ?= $(USERPROFILE)
else
  RM = rm -rf
  MKDIR = mkdir -p
endif

# ---------------------------------------------------------------------------
# Coverage — single source of truth is coverage-thresholds.json
# See REPO-STANDARDS-SPEC [COVERAGE-THRESHOLDS-JSON].
# ---------------------------------------------------------------------------
COVERAGE_THRESHOLDS_FILE := coverage-thresholds.json

help: ## Show this help
	@grep -hE '^[a-zA-Z-]+.*##' $(MAKEFILE_LIST) | \
		awk -F':.*##' '{printf "  \033[36m%-12s\033[0m%s\n", $$1, $$2}'

# =============================================================================
# Standard Targets ([MAKE-TARGETS])
# =============================================================================

build: ## Compile the dmx binary in release mode (CARGO_TARGET=<triple> to cross-compile)
	cargo build $(CRATE) --release $(if $(CARGO_TARGET),--target $(CARGO_TARGET),)

test: ## Fail-fast tests + coverage + per-component threshold enforcement ([TEST-RULES])
	@# Every component that ships is measured here, because a threshold nobody
	@# computes is not a gate. The suites themselves also run under `extension`,
	@# `website` and `dart-package`, which do the packaging, analysis and e2e
	@# work this target has no business doing — this is the coverage pass.
	rustup component add llvm-tools-preview 2>/dev/null || true
	cargo llvm-cov $(CRATE) --workspace --all-targets --lcov --output-path $(CURDIR)/lcov.info
	$(MAKE) _coverage_dart _coverage_extension _coverage_website
	$(MAKE) _coverage_check

# Private: the three non-Rust coverage producers. Each writes the LCOV path its
# component names in coverage-thresholds.json, and nothing else.

_coverage_dart:
	@# `--report-on=lib` keeps the published library the subject: without it the
	@# test files count themselves and the number stops meaning anything.
	cd $(DMX_PACKAGE_DIR) && dart pub get && dart test --coverage=.coverage
	cd $(DMX_PACKAGE_DIR) && dart run coverage:format_coverage --lcov \
		--in=.coverage --out=lcov.info --report-on=lib \
		--packages=.dart_tool/package_config.json

_coverage_extension:
	@# extension.js is absent from this measure on purpose — it requires the
	@# `vscode` module and cannot load outside the editor host, so `make vsix-e2e`
	@# is what proves it. Test files are excluded so the suite cannot cover itself.
	cd $(EXTENSION_DIR) && npm install --no-audit --no-fund
	cd $(EXTENSION_DIR) && node --test --experimental-test-coverage \
		--test-coverage-exclude='test/**' \
		--test-reporter=lcov --test-reporter-destination=lcov.info \
		--test-reporter=spec --test-reporter-destination=stdout 'test/*.test.js'

_coverage_website: wasm
	@# The WASM module is an INPUT to this suite, not an artifact of it:
	@# wasm-samples.test.ts imports src/dmx/target/wasm-node/dmx_wasm.js at module
	@# load, so vitest cannot even collect the file without it. It is ignored
	@# build output, so a clean checkout has none — which made this pass on any
	@# machine that had ever built the playground and fail on every fresh one,
	@# including CI, where `make test` runs long before `make website`.
	@#
	@# `--coverage.all` is load-bearing: v8 reports only files a test imported,
	@# so without it playground.ts is invisible and the number reads 100%.
	cd $(WEBSITE_DIR) && npm ci
	cd $(WEBSITE_DIR) && npx vitest run --coverage --coverage.all \
		--coverage.include='src/**/*.ts' --coverage.exclude='src/**/*.test.ts' \
		--coverage.reporter=lcovonly --coverage.reportsDirectory=.coverage
	cp $(WEBSITE_DIR)/.coverage/lcov.info $(WEBSITE_DIR)/lcov.info

lint: ## Clippy with warnings denied (read-only — never formats)
	cargo clippy $(CRATE) --all-targets -- -D warnings

fmt: ## Format code in-place. Pass CHECK=1 for read-only check (CI use)
	cargo fmt $(CRATE) --all$(if $(CHECK), --check,)
	@# Only HAND-WRITTEN Dart is formatted. `examples/storefront/lib`,
	@# `examples/dmx_sqlite_example/lib`, $(GOLDEN_DIR) and
	@# $(TD_GOLDEN_DIR)/lib hold dmx OUTPUT —
	@# formatting them would rewrite the very bytes the golden tests assert on.
	@# `--output none` matters: plain `dart format --set-exit-if-changed` still
	@# REWRITES the files it checks, which is not a check.
	dart format --language-version=$(DART_LANGUAGE_VERSION)$(if $(CHECK), --output none --set-exit-if-changed,) $(DART_HAND_WRITTEN)

clean: ## Remove Rust and Dart build artifacts
	cargo clean $(CRATE)
	$(RM) $(EXAMPLE_DIR)/.dart_tool $(TD_GOLDEN_DIR)/.dart_tool $(CORPUS_DIR) $(WEBSITE_DIR)/dist $(WEBSITE_DIR)/pkg lcov.info
	@# One per component, plus the raw hit data the two of them are formatted from.
	$(RM) $(DMX_PACKAGE_DIR)/lcov.info $(DMX_PACKAGE_DIR)/.coverage \
		$(EXTENSION_DIR)/lcov.info $(WEBSITE_DIR)/lcov.info $(WEBSITE_DIR)/.coverage

ci: ## Everything CI runs, in the order ci.yml runs it (full CI simulation)
	@# Steps, not prerequisites: `-j` would let prerequisites run in any order,
	@# and a CI simulation that reorders the pipeline is not one — same reason as
	@# `rebuild` below. The SET is ci.yml's too, `fmt CHECK=1` and `deslop`
	@# included: `make ci` is the gate submit-pr refuses to open a PR without, so
	@# a check that runs there and not here is a check that fails after the PR is
	@# already open.
	$(MAKE) fmt CHECK=1
	$(MAKE) lint
	$(MAKE) version-check
	$(MAKE) deslop
	$(MAKE) test
	$(MAKE) dart-package corpus example example-sqlite example-openapi extension
	$(MAKE) vsix-e2e
	$(MAKE) website
	$(MAKE) build

setup: ## Post-create dev environment setup
	rustup component add llvm-tools-preview
	rustup target add wasm32-unknown-unknown
	@# Bare `cargo`: these install named crates from the registry and take no
	@# manifest — pointing them at src/dmx/Cargo.toml is an error, not a no-op.
	cargo install cargo-llvm-cov
	cargo install wasm-bindgen-cli --version $(WASM_BINDGEN_VERSION) --locked
	cd $(DMX_PACKAGE_DIR) && dart pub get
	@# tests/golden is its own package purely so the editor's analyzer resolves
	@# package:dmx against the samples in place. `make corpus` still
	@# analyzes the generated copies under $(CORPUS_DIR).
	cd $(GOLDEN_DIR) && dart pub get
	cd $(WEBSITE_DIR) && npm ci && npx playwright install chromium
	@echo "==> Setup complete. Run 'make ci' to validate."

# Private: asserts every component's measured line coverage >= its own
# threshold in coverage-thresholds.json. Never a public target
# ([COVERAGE-THRESHOLDS-JSON]).
#
# One loop, one comparison, one report line per component — a per-language copy
# of this arithmetic is how the thresholds drifted apart in the first place.
#
# A component whose LCOV is MISSING is a failure, never a skip: the whole point
# of the split is that an unmeasured component cannot read as a covered one. It
# is also why this reports every component before exiting rather than dying on
# the first — one run should tell you everything that is under water.
_coverage_check:
	@if [ ! -f "$(COVERAGE_THRESHOLDS_FILE)" ]; then echo "FAIL: $(COVERAGE_THRESHOLDS_FILE) not found"; exit 1; fi; \
	failed=0; \
	for name in $$(jq -r '.components | keys[]' "$(COVERAGE_THRESHOLDS_FILE)"); do \
	  threshold=$$(jq -r --arg n "$$name" '.components[$$n].threshold' "$(COVERAGE_THRESHOLDS_FILE)"); \
	  lcov=$$(jq -r --arg n "$$name" '.components[$$n].lcov' "$(COVERAGE_THRESHOLDS_FILE)"); \
	  if [ ! -f "$$lcov" ]; then \
	    echo "FAIL: $$name — no coverage at $$lcov; it was never measured"; \
	    failed=1; continue; \
	  fi; \
	  LH=$$(grep '^LH:' "$$lcov" | awk -F: '{sum+=$$2} END{print sum+0}'); \
	  LF=$$(grep '^LF:' "$$lcov" | awk -F: '{sum+=$$2} END{print sum+0}'); \
	  if [ "$$LF" -eq 0 ]; then echo "FAIL: $$name — no lines in $$lcov"; failed=1; continue; fi; \
	  PCT=$$(awk "BEGIN{printf \"%.1f\", $$LH/$$LF*100}"); \
	  PCT_INT=$$(awk "BEGIN{printf \"%d\", $$LH/$$LF*100}"); \
	  if [ "$$PCT_INT" -lt "$$threshold" ]; then \
	    printf 'FAIL: %-18s %s%% < %s%% (%s/%s lines)\n' "$$name" "$$PCT" "$$threshold" "$$LH" "$$LF"; \
	    failed=1; \
	  else \
	    printf 'OK:   %-18s %s%% >= %s%% (%s/%s lines)\n' "$$name" "$$PCT" "$$threshold" "$$LH" "$$LF"; \
	  fi; \
	done; \
	if [ "$$failed" -ne 0 ]; then echo "FAIL: coverage below threshold — raise the tests, never the threshold"; exit 1; fi

# =============================================================================
# Repo-Specific Targets
#
# Owned by this repo, NOT part of the standard vocabulary. Preserved verbatim.
# =============================================================================

# --- Duplication gate [CI-DESLOP] --------------------------------------------

deslop: ## Duplication gate — the same check, against the same committed budget, CI runs
	@# A missing deslop is a FAILURE here, never a skip. A green `make ci` that
	@# quietly omitted this gate is exactly how duplication reached 10.48%
	@# against a threshold of 4.5% with nobody seeing it, and the reading that
	@# hid it came from the MCP server, which does not read .deslop.toml. Only
	@# the CLI computes the number the build tanks on.
	@command -v deslop >/dev/null 2>&1 || { \
	  echo "FAIL: deslop is not installed, so the duplication gate cannot run."; \
	  echo "      CI pins the version in .github/workflows/ci.yml — install that one:"; \
	  echo "      https://github.com/Nimblesite/Deslop/releases"; \
	  exit 1; }
	@# --output moves BOTH the reports and the timestamped log, which the CLI
	@# writes next to them. Without it 0.5.1 defaults to the working directory
	@# and litters the repository root with four files per run.
	deslop . --output .deslop/deslop-report

# --- Versioning [release.version] --------------------------------------------
# THE TAG IS THE VERSION. Every package file in the tree carries the placeholder
# below and is never bumped by hand; a release stamps what its tag names into
# its own checkout and publishes that. Nothing here commits and nothing here
# pushes — the stamped tree is a pure function of (tagged commit, tag), so
# `make version VERSION=1.2.3` on the tagged commit reproduces what shipped.
#
# Three files are stamped and each is written by the toolchain that owns its
# format, because no structured file in this repository is edited by pattern:
#
#   src/editors/vscode/package.json  JSON.parse/stringify   `make version`
#   src/dart_packages/dmx/pubspec.yaml  package:yaml span   `make dart-package-version`
#   src/dmx/Cargo.toml               NOT rewritten at all — the crate reads
#                                    DMX_VERSION at compile time, exported below
VERSION ?= 0.0.0
export DMX_VERSION := $(VERSION)

# The binary that regenerates the Dart package's version constant. Overridable
# so the release can point at the one it already built and proved, instead of
# paying for a second compile of the same commit.
DMX ?= $(CURDIR)/$(RELEASE_DIR)/$(DMX_BIN)

version: ## Stamp the VSIX manifest and changelogs with VERSION (default: the placeholder)
	node scripts/version.mjs --stamp $(VERSION)

version-check: ## Prove every package file still carries the placeholder, ready to stamp
	node --test scripts/version.test.mjs
	@# The pubspec version is read through `dart pub deps --json`, so pub parses
	@# its own file and nothing here reads YAML. That needs the package RESOLVED,
	@# and this target runs before `dart-package` does — so it resolves it rather
	@# than depending on a developer having happened to.
	cd $(DMX_PACKAGE_DIR) && dart pub get
	node scripts/version.mjs --check

release-check: version-check ## Prove TAG=v1.2.3 can be stamped into this tree and published
	@if [ -z "$(TAG)" ]; then echo "usage: make release-check TAG=v1.2.3"; exit 1; fi
	node scripts/version.mjs --tag $(TAG)

dart-package-version: ## Stamp the Dart package's pubspec with VERSION and regenerate its constant
	@# `lib/src/version.dart` is GENERATED from the pubspec by this package's own
	@# macro worker, so stamping the pubspec without regenerating would publish a
	@# package whose `DmxPackage.version` names the release before it.
	@test -x "$(DMX)" || { echo "no dmx binary at $(DMX) — run 'make build', or pass DMX=<path>"; exit 1; }
	cd $(DMX_PACKAGE_DIR) && dart pub get && dart run tool/stamp_version.dart $(VERSION)
	cd $(DMX_PACKAGE_DIR) && $(DMX) build lib --insert-regions

wasm: ## Compile the real dmx generator for browsers
	rustup target add wasm32-unknown-unknown
	node scripts/build-wasm.mjs

wasm-test: wasm ## Execute the compiled generator in Node
	node scripts/wasm-smoke.cjs

website: wasm-test ## Test and build the website + WASM playground
	cd $(WEBSITE_DIR) && npm ci && npm test && npm run build && npm run test:e2e

dart-package: ## Analyze and test the hand-written dmx Dart package
	@# The examples analyze GENERATED Dart; this analyzes the hand-written
	@# package they all depend on, which no other target covers. It also runs
	@# the Dart half of the casing corpus gate — `matches_the_shared_casing_corpus`
	@# in src/casing.rs proves the Rust side against tests/casing_corpus.json,
	@# and `test/casing_test.dart` proves the Dart port against the same file
	@# [context.helpers]. Only one of those two ran under `make test`, so a Dart
	@# port that drifted from `casing.rs` reached CI green until this target
	@# existed.
	@# pub publishes only what git tracks, so the licence cannot be staged into
	@# the package the way `make vsix` stages it — the copy has to be committed.
	@# It is proven identical here instead, so a second copy cannot drift.
	cmp LICENSE $(DMX_PACKAGE_DIR)/LICENSE
	cd $(DMX_PACKAGE_DIR) && dart pub get && dart analyze --fatal-infos && dart test
	@# A package that only analyzes is a package nobody has actually consumed.
	cd $(DMX_PACKAGE_DIR) && dart run example/example.dart

dart-package-publish: dart-package ## Prove the pub archive is publishable (needs a clean tree)
	@# Deliberately not part of `ci`: pub warns on any modified checked-in file,
	@# so this passes only from a clean checkout — which is what a tag is.
	cd $(DMX_PACKAGE_DIR) && dart pub publish --dry-run

example: ## Generate the example — annotated Dart and its typeDiagram definitions — analyze it, run its checks
	@# One invocation for both backends. Annotated Dart is generated INTO, and a
	@# `.td` definition resolves its outputs against the package it belongs to
	@# [typediagram.output] — so neither depends on where this runs from, unlike
	@# the macro-worker examples below, whose workers are found from the cwd.
	cargo run $(CRATE) --quiet -- build $(EXAMPLE_DIR)/lib $(EXAMPLE_DIR)/models --insert-regions
	cd $(EXAMPLE_DIR) && dart pub get && dart analyze --fatal-infos && dart test

EXAMPLE run-example: example

SQLITE_EXAMPLE_DIR := examples/dmx_sqlite_example

example-sqlite: ## Build the db, generate from it with the Dart macro, prove it [dartmacros]
	@# Redirected, not passed as an argument: schema.sql opens with a `--`
	@# comment, which sqlite3 would read as a command-line option.
	$(RM) $(SQLITE_EXAMPLE_DIR)/tool/dmx/app.db
	sqlite3 $(SQLITE_EXAMPLE_DIR)/tool/dmx/app.db < $(SQLITE_EXAMPLE_DIR)/tool/dmx/schema.sql
	cargo build $(CRATE) --quiet
	@# Run FROM the package: `tool/dmx/macros.dart` is found relative to the
	@# working directory [dartmacros.discovery], and so is the `db` argument.
	@# Driven from the repo root the worker is simply absent and every
	@# `@dmx('sqliteSchema')` stays inert — a silent no-op, not a build.
	@# `dart pub get` FIRST, like example-openapi: the worker is a Dart program
	@# importing package:dmx, so with no `.dart_tool` every symbol it uses is
	@# unresolved and it dies on startup as DMX7002. That only shows up on a
	@# clean checkout — anywhere the packages were already resolved it passes.
	cd $(SQLITE_EXAMPLE_DIR) && dart pub get && $(CURDIR)/$(TARGET_DIR)/debug/dmx build lib --insert-regions
	cd $(SQLITE_EXAMPLE_DIR) && dart analyze --fatal-infos && dart test

OPENAPI_EXAMPLE_DIR := examples/dmx_openapi_example

example-openapi: ## Generate a typed API client from an OpenAPI document, prove it [dartmacros.render]
	cargo build $(CRATE) --quiet
	@# Run FROM the package, like every macro worker [dartmacros.discovery]: the
	@# worker, its OpenAPI document, and its Mustache templates are all found
	@# relative to it. Driven from the repo root the worker is simply absent and
	@# `@dmx('openApiClient')` stays inert.
	cd $(OPENAPI_EXAMPLE_DIR) && dart pub get && $(CURDIR)/$(TARGET_DIR)/debug/dmx build lib --insert-regions
	@# `dart test` here is hermetic: the live suite is in test_live/, which a
	@# bare run does not look in, so this gate never depends on somebody else's
	@# uptime. `make example-openapi-live` is how you run it against the API.
	cd $(OPENAPI_EXAMPLE_DIR) && dart analyze --fatal-infos && dart test

example-openapi-live: example-openapi ## Run the generated client against the real API (needs network)
	cd $(OPENAPI_EXAMPLE_DIR) && dart test test_live

dev: ## Insert missing regions, then watch (what the VS Code auto-start task runs)
	@# `watch` deliberately refuses --insert-regions, so a class annotated before
	@# the session starts gets its divider here. A leading `-` keeps a source that
	@# is currently invalid Dart from stopping the watcher coming up behind it.
	-cargo run $(CRATE) --quiet -- build $(EXAMPLE_DIR)/lib --insert-regions
	cargo run $(CRATE) --quiet -- watch $(EXAMPLE_DIR)/lib

watch: ## Regenerate the example when its Dart sources change
	cargo run $(CRATE) --quiet -- watch $(EXAMPLE_DIR)/lib

golden: ## Regenerate the golden files after a deliberate shape change
	UPDATE_GOLDEN=1 cargo test $(CRATE) --test golden
	cargo test $(CRATE) --test golden

# --- VS Code extension [editor.extension] ------------------------------------
EXTENSION_DIR := src/editors/vscode

# The platform the VSIX is FOR. Defaults to the host, so `make vsix` on a laptop
# packages for that laptop; the release matrix overrides it, together with
# CARGO_TARGET when the binary is cross-compiled ([MAKE-IDE-EXT-PARITY]).
ifeq ($(OS),Windows_NT)
  HOST_VSIX_TARGET := win32-x64
else
  VSIX_ARCH := $(patsubst x86_64,x64,$(patsubst aarch64,arm64,$(shell uname -m)))
  HOST_VSIX_TARGET := $(shell uname -s | tr '[:upper:]' '[:lower:]')-$(VSIX_ARCH)
endif
VSIX_TARGET ?= $(HOST_VSIX_TARGET)

# The binary's name follows the platform it RUNS on, not the one building it.
ifeq (,$(findstring win32,$(VSIX_TARGET)))
  DMX_BIN := dmx
else
  DMX_BIN := dmx.exe
endif

# Empty means "build for the host", and cargo puts that straight in
# target/release. A cross build lands one directory deeper, under its triple.
CARGO_TARGET ?=
RELEASE_DIR := $(TARGET_DIR)/$(if $(CARGO_TARGET),$(CARGO_TARGET)/,)release

extension: ## Tokenize the real templates with the real VS Code grammar engine
	cd $(EXTENSION_DIR) && npm install --no-audit --no-fund && npm test

vsix: build extension version ## Package the extension for VSIX_TARGET, binary included
	@# One VSIX per platform, each carrying one binary: the marketplace serves
	@# the matching one, so nobody downloads a toolchain to get a watcher
	@# [editor.extension.binary].
	@$(MKDIR) $(EXTENSION_DIR)/bin
	cp $(RELEASE_DIR)/$(DMX_BIN) $(EXTENSION_DIR)/bin/$(DMX_BIN)
	@# The bundle carries the licence it ships under, and the repo root is the
	@# one copy anybody edits.
	cp LICENSE $(EXTENSION_DIR)/LICENSE
	cd $(EXTENSION_DIR) && npx vsce package --target $(VSIX_TARGET) \
		--out $(CURDIR)/$(TARGET_DIR)/dmx-$(VSIX_TARGET).vsix

vsix-e2e: vsix ## Drive the packaged VSIX in a real VS Code, engine rebuilt and all
	@# The artifact itself, not the tree it was packaged from: the suite unpacks
	@# target/dmx-$(VSIX_TARGET).vsix and boots VS Code on it, so a .vscodeignore
	@# line that eats the binary fails HERE, not in a user's editor
	@# [editor.extension.e2e]. `vsix` depends on `build`, so the engine the
	@# bundle carries is rebuilt from this tree first.
	cd $(EXTENSION_DIR) && DMX_VSIX=$(CURDIR)/$(TARGET_DIR)/dmx-$(VSIX_TARGET).vsix npm run e2e

vsix-universal: extension version ## Package the no-binary VSIX every other platform falls back to
	@# The marketplace serves this one to any platform the matrix above does not
	@# cover, so the extension is installable everywhere and resolves `dmx` from
	@# PATH there [editor.extension.binary]. It must carry NO binary: a bundle
	@# with one would hand every unmatched platform the wrong architecture.
	$(RM) $(EXTENSION_DIR)/bin
	cp LICENSE $(EXTENSION_DIR)/LICENSE
	@# `vsce` opens the --out path, it does not create the directory holding it.
	@# `vsix` gets that directory for free from `build`; this target carries no
	@# binary and so builds nothing, which on a fresh checkout — a release
	@# runner, every time — means target/ does not exist and packaging dies with
	@# ENOENT after the bundle was assembled.
	@$(MKDIR) $(CURDIR)/$(TARGET_DIR)
	cd $(EXTENSION_DIR) && npx vsce package --out $(CURDIR)/$(TARGET_DIR)/dmx-universal.vsix

# Full clean rebuild-and-reinstall cycle for the VS Code extension
# ([MAKE-IDE-EXT]): uninstall → clean → rebuild+package → install. `vsix` above
# is the SINGLE packaging recipe — this target and the release path both route
# through it, so the bundle installed locally is the bundle that ships
# ([MAKE-IDE-EXT-PARITY]).
rebuild-install-vsix: _vsix_uninstall _vsix_clean vsix _vsix_install ## Uninstall, rebuild, repackage and reinstall the VS Code extension

_vsix_uninstall:
	-code --uninstall-extension nimblesite.dmx

_vsix_clean:
	$(RM) $(EXTENSION_DIR)/bin $(TARGET_DIR)/dmx-$(VSIX_TARGET).vsix

_vsix_install:
	code --install-extension $(TARGET_DIR)/dmx-$(VSIX_TARGET).vsix

rebuild: ## From nothing: clean, rebuild the binary, and package the VSIX
	@# Prerequisites would let `-j` start the build before the clean finished,
	@# and a from-scratch target that races itself is not one. The steps are
	@# sub-makes so the order is the recipe, not the scheduler's opinion.
	$(MAKE) clean
	$(MAKE) _vsix_clean
	$(MAKE) vsix
	@echo "==> $(TARGET_DIR)/dmx-$(VSIX_TARGET).vsix — 'make rebuild-install-vsix' to install it"

corpus: ## Generate every golden sample and prove it is valid Dart
	@$(RM) $(CORPUS_DIR) && $(MKDIR) $(CORPUS_DIR)/lib
	@cp $(GOLDEN_DIR)/*.dart $(CORPUS_DIR)/lib/
	@printf 'name: dmx_corpus\nenvironment:\n  sdk: ^3.0.0\ndependencies:\n  dmx: ^0.3.0\n' > $(CORPUS_DIR)/pubspec.yaml
	cargo run $(CRATE) --quiet -- build $(CORPUS_DIR)/lib --insert-regions
	cd $(CORPUS_DIR) && dart pub get && dart analyze --fatal-infos
	@# The typeDiagram corpus is generated the other way round: no annotated
	@# Dart at all, just `tests/typediagram/corpus/*.td` rendered through the
	@# canonical model template dmx ships [typediagram.canonical]. `cargo test
	@# --test typediagram_golden` proves the committed files are what the binary
	@# writes; this proves they are Dart the analyzer accepts, which is the
	@# half a byte comparison cannot do.
	cd $(TD_GOLDEN_DIR) && dart pub get && dart analyze --fatal-infos
