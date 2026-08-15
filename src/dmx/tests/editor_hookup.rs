//! The watcher is only useful if something starts it [execution.modes], [cli].
//!
//! A correct `dmx watch` that nobody launches is indistinguishable, from the
//! editor, from a broken one: you delete a generated member, it stays deleted.
//! These tests cover the wiring rather than the watching — that the workspace
//! auto-starts the watcher, and that the command it auto-starts still exists.

// [TEST-RULES] admits `expect` in a test: a fixture that cannot be built is a
// broken test, and unwinding at the point of failure names it better than any
// `Result` plumbing would. Production code is still held to `unwrap_used` and
// `expect_used` at deny — this relaxation is `cfg(test)`-scoped on purpose.
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects
    )
)]

use json_comments::StripComments;

#[path = "support/repo.rs"]
mod repo;
use repo::{read, repo_root};

/// One of the editor's own configuration files. They are JSON with comments —
/// VS Code's dialect — and the comments in them are load-bearing documentation,
/// so they are removed by a tokeniser and the result parsed as the JSON it then
/// is. Nothing here reads structured data by pattern.
fn editor_json(relative: &str) -> serde_json::Value {
    serde_json::from_reader(StripComments::new(read(relative).as_bytes()))
        .unwrap_or_else(|e| panic!("{relative} is not valid JSON-with-comments: {e}"))
}

/// Every task the workspace defines, as (label, command).
fn tasks() -> Vec<(String, String)> {
    editor_json(".vscode/tasks.json")["tasks"]
        .as_array()
        .expect(".vscode/tasks.json declares no tasks")
        .iter()
        .map(|task| {
            let label = task["label"].as_str().unwrap_or("<unlabelled>").to_owned();
            let command = task["command"]
                .as_str()
                .unwrap_or_else(|| panic!("the `{label}` task runs no command"))
                .to_owned();
            (label, command)
        })
        .collect()
}

/// Makefile target names, i.e. every line starting `name:` at column zero.
fn make_targets(makefile: &str) -> Vec<String> {
    makefile
        .lines()
        .filter(|line| !line.starts_with([' ', '\t', '#', '.']))
        .filter_map(|line| line.split_once(':'))
        .map(|(name, _)| name.trim().to_owned())
        .filter(|name| !name.is_empty() && !name.contains('='))
        .collect()
}

/// Every `make <target>` a task invokes must actually exist, so renaming a
/// target cannot silently disarm the auto-start.
#[test]
fn every_make_target_the_tasks_invoke_exists() {
    let targets = make_targets(&read("Makefile"));
    let invoked: Vec<String> = tasks()
        .into_iter()
        .filter_map(|(_, command)| command.split_whitespace().nth(1).map(str::to_owned))
        .collect();

    assert!(!invoked.is_empty(), "no make target is invoked by any task");
    for name in invoked {
        assert!(
            targets.contains(&name),
            "tasks.json runs `make {name}`, which is not a Makefile target; \
             targets are {targets:?}"
        );
    }
}

/// [repo.layout]: every task goes through `make`, which keeps the Makefile the
/// one place that knows where anything in this repository lives.
///
/// The build task used to run `cargo run` from the repository root directly.
/// Moving the crate to `src/dmx` left the root with no `Cargo.toml`, so pressing
/// it stopped generating the example and started printing `could not find
/// Cargo.toml` — while the target it should have been calling worked fine.
#[test]
fn every_editor_task_runs_through_make() {
    let tasks = tasks();
    assert!(
        tasks.len() >= 3,
        "the workspace defines {} tasks, which is fewer than it ships",
        tasks.len()
    );
    for (label, command) in tasks {
        assert_eq!(
            command.split_whitespace().next(),
            Some("make"),
            "the `{label}` task runs `{command}` instead of a make target — only the \
             Makefile knows where this repository keeps its crate and its packages"
        );
    }
}

/// The whole point: opening the workspace starts the watcher unprompted.
#[test]
fn the_workspace_starts_the_watcher_on_folder_open() {
    let configuration = editor_json(".vscode/tasks.json");
    let watcher = configuration["tasks"]
        .as_array()
        .expect(".vscode/tasks.json declares no tasks")
        .iter()
        .find(|task| task["runOptions"]["runOn"] == "folderOpen")
        .expect(".vscode/tasks.json has no folderOpen task, so nothing starts the watcher");
    assert_eq!(
        watcher["isBackground"],
        serde_json::Value::Bool(true),
        "the watcher task must be a background task; it never exits"
    );
}

/// A suite nobody can see is a suite nobody trusts.
///
/// The Testing panel lists Dart tests out of the box because the Dart extension
/// contributes them. rust-analyzer contributes Rust tests only when its test
/// explorer is switched on, so without this the panel shows one Dart file and
/// the Rust suite looks like it does not exist.
#[test]
fn the_workspace_shows_rust_tests_in_the_testing_panel() {
    assert_eq!(
        editor_json(".vscode/settings.json")["rust-analyzer.testExplorer"],
        serde_json::Value::Bool(true),
        ".vscode/settings.json does not enable the rust-analyzer test explorer, \
         so the Testing panel hides every Rust test"
    );
    assert!(
        editor_json(".vscode/extensions.json")["recommendations"]
            .as_array()
            .expect(".vscode/extensions.json recommends nothing")
            .iter()
            .any(|extension| extension == "rust-lang.rust-analyzer"),
        "the test explorer setting is inert unless rust-analyzer is recommended"
    );
}

/// `dmx watch` refuses `--insert-regions`, so a class annotated while the editor
/// was closed would otherwise never get a divider. `dev` runs the build first.
#[test]
fn the_dev_target_inserts_regions_before_watching() {
    let makefile = read("Makefile");
    let Some((_, dev)) = makefile.split_once("\ndev:") else {
        panic!("Makefile has no `dev` target for the editor to start");
    };
    // Recipe lines only: a `@#` comment inside the recipe mentions both of the
    // things being ordered here, and would match either way round.
    let body: String = dev
        .lines()
        .skip(1)
        .take_while(|line| line.starts_with([' ', '\t', '@', '-']))
        .filter(|line| !line.trim_start().starts_with(['#', '@']))
        .collect();
    let insert = body
        .find("--insert-regions")
        .expect("`dev` never inserts regions, so new @dmx('model') classes stay empty");
    let watch = body
        .find("watch")
        .expect("`dev` never starts the watcher, so saves do not regenerate");
    assert!(
        insert < watch,
        "`dev` must insert regions before watching, not after"
    );
}

// ---------------------------------------------------------------------------
// The VS Code extension [editor.extension]
//
// A manifest is a pile of strings that name files, commands, settings and
// scopes, and nothing in the packaging step checks that any of them still point
// at anything. These tests are that check: rename a file, a command, or a
// setting on one side of the wiring and the suite says so, rather than a user
// discovering it as an extension that activates and does nothing.
// ---------------------------------------------------------------------------

const EXTENSION_DIR: &str = "src/editors/vscode";

fn manifest() -> serde_json::Value {
    serde_json::from_str(&read(&format!("{EXTENSION_DIR}/package.json")))
        .expect("the extension manifest is not valid JSON")
}

fn extension_source() -> String {
    read(&format!("{EXTENSION_DIR}/extension.js"))
}

/// Every string the JSON at `pointer` yields for `key`, across the array.
fn strings(manifest: &serde_json::Value, pointer: &str, key: &str) -> Vec<String> {
    manifest
        .pointer(pointer)
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("the manifest has no array at {pointer}"))
        .iter()
        .filter_map(|entry| entry.get(key))
        .filter_map(serde_json::Value::as_str)
        .map(str::to_owned)
        .collect()
}

/// Everything `extension.js` passes to `needle('…`, e.g. every command id it
/// registers or every setting name it reads.
fn arguments(source: &str, needle: &str) -> Vec<String> {
    source
        .match_indices(needle)
        .filter_map(|(at, _)| source[at + needle.len()..].split('\'').next())
        .map(str::to_owned)
        .collect()
}

/// [editor.extension]: a manifest that names a file the package does not carry
/// is an extension that fails on activation, in a user's editor, once.
#[test]
fn every_file_the_extension_manifest_names_is_present() {
    let manifest = manifest();
    let mut declared = vec![
        manifest["main"]
            .as_str()
            .expect("the manifest has no entry point")
            .to_owned(),
    ];
    declared.extend(strings(
        &manifest,
        "/contributes/languages",
        "configuration",
    ));
    declared.extend(strings(&manifest, "/contributes/grammars", "path"));

    assert!(
        declared.len() >= 4,
        "declared too little to be the manifest"
    );
    for relative in declared {
        let path = repo_root()
            .join(EXTENSION_DIR)
            .join(relative.trim_start_matches("./"));
        assert!(
            path.is_file(),
            "the manifest names {}, which does not exist",
            path.display()
        );
    }
}

/// [editor.extension]: the palette lists what the manifest contributes and the
/// extension answers what it registered. Either half alone is a dead menu item
/// or an unreachable feature.
#[test]
fn contributed_commands_and_registered_commands_are_the_same_set() {
    let mut contributed = strings(&manifest(), "/contributes/commands", "command");
    let mut registered = arguments(&extension_source(), "registerCommand('");
    contributed.sort();
    registered.sort();
    assert!(
        !contributed.is_empty(),
        "the extension contributes no commands"
    );
    assert_eq!(
        contributed, registered,
        "contributed and registered commands differ"
    );
}

/// [editor.extension]: a setting the code reads but the manifest never declares
/// cannot be discovered, cannot be set in the UI, and silently takes its
/// hard-coded default forever.
#[test]
fn every_setting_the_extension_reads_is_declared() {
    let declared: Vec<String> = manifest()
        .pointer("/contributes/configuration/properties")
        .and_then(serde_json::Value::as_object)
        .expect("the manifest declares no configuration")
        .keys()
        .map(|key| key.trim_start_matches("dmx.").to_owned())
        .collect();
    let used = arguments(&extension_source(), "settings.get('");

    assert!(!used.is_empty(), "the extension reads no settings at all");
    for name in used {
        assert!(
            declared.contains(&name),
            "the extension reads `dmx.{name}`, which the manifest never declares; \
             declared: {declared:?}"
        );
    }
}

/// [editor.extension.binary]: the packaging step stages the binary at one path
/// and the extension looks for it at one path. They are the same path or the
/// bundle is decoration.
#[test]
fn the_extension_looks_for_the_binary_where_packaging_puts_it() {
    assert!(
        extension_source().contains("path.join('bin', BINARY)"),
        "the extension no longer looks inside its own `bin/` for the binary"
    );
    let makefile = read("Makefile");
    let Some((_, vsix)) = makefile.split_once("\nvsix:") else {
        panic!("the Makefile has no `vsix` target, so nothing packages the binary");
    };
    let recipe: String = vsix
        .lines()
        .skip(1)
        .take_while(|line| line.starts_with([' ', '\t', '@', '-']))
        .collect();
    assert!(
        recipe.contains("$(EXTENSION_DIR)/bin/$(DMX_BIN)"),
        "`make vsix` no longer stages the binary in the extension's `bin/`:\n{recipe}"
    );
    assert!(
        recipe.contains("cp LICENSE $(EXTENSION_DIR)/LICENSE"),
        "`make vsix` no longer stages the licence, so the bundle ships without \
         the terms it is published under:\n{recipe}"
    );
    assert!(
        recipe.contains("--target $(VSIX_TARGET)"),
        "`make vsix` packages one VSIX for every platform, which can only carry \
         one platform's binary:\n{recipe}"
    );
}

/// [editor.template-highlighting]: the tags are injected into the language's own
/// scope, at a priority that outranks the Dart rules underneath them. Get the
/// scope name wrong on either side and the tags simply never highlight.
#[test]
fn the_mustache_tags_are_injected_into_the_mustache_scope() {
    let manifest = manifest();
    let grammars = manifest
        .pointer("/contributes/grammars")
        .and_then(serde_json::Value::as_array)
        .expect("the manifest contributes no grammars");

    let language = grammars
        .iter()
        .find(|grammar| grammar["language"] == "mustache")
        .expect("no grammar is bound to the mustache language");
    let scope = language["scopeName"]
        .as_str()
        .expect("grammar has no scope");

    let injection = grammars
        .iter()
        .find(|grammar| grammar.get("injectTo").is_some())
        .expect("nothing injects the mustache tags");
    assert!(
        injection["injectTo"]
            .as_array()
            .is_some_and(|targets| targets.iter().any(|target| target == scope)),
        "the tag injection does not target `{scope}`"
    );

    let path = injection["path"].as_str().expect("injection has no path");
    let grammar: serde_json::Value = serde_json::from_str(&read(&format!(
        "{EXTENSION_DIR}/{}",
        path.trim_start_matches("./")
    )))
    .expect("the injection grammar is not valid JSON");
    assert_eq!(
        grammar["injectionSelector"],
        serde_json::Value::String(format!("L:{scope}")),
        "the injection must outrank the host grammar, or a tag inside a Dart \
         string literal reads as string"
    );
}

/// [editor.extension]: the extension ships a binary built from this crate, so a
/// version that says otherwise is a support question waiting to happen.
#[test]
fn the_extension_version_matches_the_crate_version() {
    let crate_version = read("src/dmx/Cargo.toml")
        .lines()
        .find_map(|line| line.strip_prefix("version = "))
        .map(|value| value.trim().trim_matches('"').to_owned())
        .expect("Cargo.toml has no version");
    assert_eq!(
        manifest()["version"],
        serde_json::Value::String(crate_version),
        "the extension and the binary it carries claim different versions"
    );
}

/// [editor.extension]: the marketplace shows the licence the manifest names.
/// Naming one the repository does not ship is the kind of wrong that only ever
/// surfaces as a legal question.
#[test]
fn the_extension_declares_the_licence_the_repository_ships() {
    let licence = read("LICENSE");
    assert!(
        licence.starts_with("BSD 3-Clause License"),
        "the repository LICENSE is not the BSD 3-Clause text"
    );
    assert_eq!(
        manifest()["license"],
        serde_json::Value::String("BSD-3-Clause".to_owned()),
        "the extension manifest names a different licence from the one shipped"
    );
    assert!(
        read("src/dmx/Cargo.toml").contains("license = \"BSD-3-Clause\""),
        "the crate does not declare the licence it is published under"
    );
    assert_eq!(
        licence,
        read("src/dart_packages/dmx/LICENSE"),
        "the published Dart package carries a different licence from the repository"
    );
}
