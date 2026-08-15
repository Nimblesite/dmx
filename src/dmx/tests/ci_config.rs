//! The repository's own tooling names paths, and nothing compiles it [repo.layout].
//!
//! A workflow, a dependabot entry and an editor task are all just strings that
//! claim a file is somewhere. Move the directory and every one of them keeps
//! claiming it, with the whole suite still green — the crate moved to `src/dmx`
//! and the extension to `src/editors/vscode`, and three of them broke at once
//! without a single test noticing.
//!
//! Where they break is what makes them worth a gate: a tag that dies in
//! `preflight` after the tag is already pushed [release], and dependency updates
//! that simply stop being opened for a package nobody is watching for silence.
//!
//! Read with the formats' own parsers, never by pattern: YAML for the workflows,
//! JSON for what a run block quotes. Only the shell inside a `run:` block is
//! tokenised here, because a shell script is not structured data.

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

use std::fs;
use std::path::{Path, PathBuf};

use yaml_rust2::{Yaml, YamlLoader};

#[path = "support/repo.rs"]
mod repo;
use repo::{read, repo_root};

/// A repository-relative path, however the file that named it spelled it: a
/// workflow writes `./.github/workflows/ci.yml`, dependabot writes
/// `/src/editors/vscode`, and a run block writes neither prefix.
fn resolve(named: &str) -> PathBuf {
    let relative = named.strip_prefix("./").unwrap_or(named);
    repo_root().join(relative.trim_start_matches('/'))
}

/// Whether the tree carries what `named` claims it does.
fn present(named: &str) -> bool {
    resolve(named).exists()
}

/// The first YAML document in a repository-relative file.
fn document(relative: &str) -> Yaml {
    YamlLoader::load_from_str(&read(relative))
        .unwrap_or_else(|e| panic!("{relative} is not valid YAML: {e}"))
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("{relative} is empty"))
}

/// Every workflow the repository ships, as (file name, parsed document).
fn workflows() -> Vec<(String, Yaml)> {
    let directory = repo_root().join(".github/workflows");
    let mut found: Vec<String> = fs::read_dir(&directory)
        .unwrap_or_else(|e| panic!("cannot list {}: {e}", directory.display()))
        .map(|entry| entry.expect("a workflow directory entry").file_name())
        .filter_map(|name| name.to_str().map(str::to_owned))
        .filter(|name| {
            Path::new(name)
                .extension()
                .is_some_and(|extension| extension == "yml" || extension == "yaml")
        })
        .collect();
    found.sort();

    assert!(
        !found.is_empty(),
        "the repository ships no workflows at all"
    );
    found
        .into_iter()
        .map(|name| {
            let parsed = document(&format!(".github/workflows/{name}"));
            (name, parsed)
        })
        .collect()
}

/// A workflow's triggers. YAML 1.1 reads a bare `on` as the boolean it spells,
/// and parsers disagree about whether YAML 1.2 still should — so both keys are
/// tried rather than assuming which one this parser produced.
fn triggers(workflow: &Yaml) -> Option<&Yaml> {
    let named = &workflow["on"];
    if !named.is_badvalue() {
        return Some(named);
    }
    workflow
        .as_hash()
        .and_then(|hash| hash.get(&Yaml::Boolean(true)))
}

/// Every job in a workflow, as (name, body).
fn jobs(workflow: &Yaml) -> Vec<(String, &Yaml)> {
    workflow["jobs"]
        .as_hash()
        .map(|hash| {
            hash.iter()
                .filter_map(|(name, body)| name.as_str().map(|n| (n.to_owned(), body)))
                .collect()
        })
        .unwrap_or_default()
}

/// Every step in a job, in the order the runner takes them. A job with no
/// `steps` is one that calls a reusable workflow instead, which is not a
/// mistake — it simply has none.
fn steps(job: &Yaml) -> impl Iterator<Item = &Yaml> {
    job["steps"].as_vec().into_iter().flatten()
}

/// Every `run:` script in a workflow, as (job name, script).
fn run_blocks(workflow: &Yaml) -> Vec<(String, String)> {
    jobs(workflow)
        .into_iter()
        .flat_map(|(name, body)| {
            steps(body)
                .filter_map(|step| step["run"].as_str())
                .map(|script| (name.clone(), script.to_owned()))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// The commands a shell line runs, as word lists. A line runs more than one:
/// `version=$(cargo metadata …)` runs `cargo`, and the naive first word of that
/// line is `version=$(cargo`, which is how a broken invocation hides. Splitting
/// on the characters that begin a nested or chained command is enough to find
/// each one's own first word, and a `#` comment is dropped rather than read.
fn commands(line: &str) -> Vec<Vec<&str>> {
    let code = line.split(" #").next().unwrap_or(line).trim();
    if code.starts_with('#') {
        return Vec::new();
    }
    code.split(['|', ';', '&', '`', '(', ')', '{', '}'])
        .map(|segment| {
            segment
                .split_whitespace()
                // `VAR=value cargo …` still runs cargo. An assignment is a
                // prefix to the command, not the command.
                .skip_while(|word| word.contains('=') && !word.starts_with('-'))
                .collect::<Vec<_>>()
        })
        .filter(|words| !words.is_empty())
        .collect()
}

// ---------------------------------------------------------------------------
// Dependabot [repo.layout]
// ---------------------------------------------------------------------------

/// The manifest an ecosystem is watched through. A directory holding none of it
/// is a directory dependabot opens nothing for.
fn manifest_for(ecosystem: &str) -> &'static str {
    match ecosystem {
        "cargo" => "Cargo.toml",
        "npm" => "package.json",
        "pub" => "pubspec.yaml",
        "github-actions" => ".github/workflows",
        other => panic!("dependabot watches `{other}`, which this test does not know how to check"),
    }
}

/// [repo.layout]: dependabot fails SILENTLY on a directory that is not there —
/// no PR, no error, no annotation, just an ecosystem that quietly stops being
/// updated. The extension's npm package moved under `src/` and this is the only
/// thing that would have said so.
#[test]
fn every_dependabot_directory_carries_the_manifest_it_is_watched_through() {
    let dependabot = document(".github/dependabot.yml");
    let updates = dependabot["updates"]
        .as_vec()
        .expect("dependabot.yml declares no updates");
    assert!(!updates.is_empty(), "dependabot watches nothing");

    for update in updates {
        let ecosystem = update["package-ecosystem"]
            .as_str()
            .expect("an update with no package-ecosystem");
        // `directory` is the single form; `directories` is the list form. Both
        // are valid and either may appear, so neither is assumed.
        let named: Vec<&str> = match update["directories"].as_vec() {
            Some(list) => list.iter().filter_map(Yaml::as_str).collect(),
            None => update["directory"].as_str().into_iter().collect(),
        };
        assert!(
            !named.is_empty(),
            "the {ecosystem} update names no directory"
        );

        for directory in named {
            assert!(
                resolve(directory).is_dir(),
                "dependabot watches {directory} for {ecosystem}, which is not a directory in \
                 this tree — the ecosystem is silently unwatched"
            );
            let manifest = manifest_for(ecosystem);
            let held = resolve(directory).join(manifest);
            assert!(
                held.exists(),
                "dependabot watches {directory} for {ecosystem}, which carries no {manifest} \
                 — nothing there is a {ecosystem} package"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Workflows [release], [repo.layout]
// ---------------------------------------------------------------------------

/// Subcommands that resolve a package from the working directory and therefore
/// need to be told where the manifest is. The complement — `install`, `new` and
/// friends — takes no manifest at all, and naming one is an error rather than a
/// no-op, which is why this is a list of what needs it rather than what does not.
const NEEDS_A_MANIFEST: &[&str] = &[
    "build", "check", "clean", "clippy", "doc", "fmt", "llvm-cov", "metadata", "package",
    "publish", "run", "test", "tree", "update",
];

/// [repo.layout]: the repository root carries no `Cargo.toml` — the crate is
/// self-contained at `src/dmx`. A workflow that runs cargo without saying so
/// fails with `could not find Cargo.toml`, and the one that did stopped every
/// release in `preflight`, after the tag was already pushed and unpushable.
///
/// This is a prohibition, so it passes on a tree that invokes cargo nowhere at
/// all — which is very nearly the tree today, the release having been moved onto
/// `scripts/version.mjs` so that one file knows where the crate is. The count is
/// what stops that from turning into a test that examines nothing: every cargo
/// invocation is counted, `install` included, so a tokeniser that stopped
/// finding them fails here rather than reporting silent approval.
#[test]
fn no_workflow_runs_cargo_without_naming_the_manifest() {
    let mut examined = 0_usize;
    for (name, workflow) in workflows() {
        for (job, script) in run_blocks(&workflow) {
            for line in script.lines() {
                for words in commands(line) {
                    let Some((&command, arguments)) = words.split_first() else {
                        continue;
                    };
                    if command != "cargo" {
                        continue;
                    }
                    examined += 1;
                    let subcommand = arguments.first().copied().unwrap_or_default();
                    if !NEEDS_A_MANIFEST.contains(&subcommand) {
                        continue;
                    }
                    assert!(
                        arguments.contains(&"--manifest-path"),
                        "{name} job `{job}` runs `cargo {subcommand}` without --manifest-path, \
                         and the repository root has no Cargo.toml:\n    {}",
                        line.trim()
                    );
                }
            }
        }
    }
    assert!(
        examined > 0,
        "no workflow was seen running cargo at all, which the workflows do — the \
         shell tokenisation has stopped finding invocations, so this test now \
         approves whatever it is handed"
    );
}

/// [repo.layout]: a path a workflow hands to a tool. Build OUTPUT is excluded —
/// `target/` holds what the run is about to produce, so it is legitimately
/// absent — and so is anything carrying an expression, which is not a path until
/// the runner expands it.
fn is_a_source_path(word: &str) -> bool {
    let cleaned = word.trim_matches(|c| c == '"' || c == '\'' || c == ',');
    cleaned.starts_with("src/")
        && !cleaned.contains("${{")
        && !cleaned.contains('*')
        && !cleaned.split('/').any(|segment| segment == "target")
}

/// [repo.layout]: every `src/…` a run block hands to a tool has to be there.
/// This is the check that generalises the release workflow's dead manifest path:
/// `npm ci --prefix src/editors/vscode` breaks exactly the same way, in exactly
/// the same commit, and reports it just as late.
#[test]
fn every_source_path_a_workflow_hands_to_a_tool_exists() {
    let mut checked = 0_usize;
    for (name, workflow) in workflows() {
        for (job, script) in run_blocks(&workflow) {
            for word in script.split_whitespace() {
                if !is_a_source_path(word) {
                    continue;
                }
                let path = word.trim_matches(|c| c == '"' || c == '\'' || c == ',');
                assert!(
                    present(path),
                    "{name} job `{job}` names {path}, which is not in this tree"
                );
                checked += 1;
            }
        }
    }
    assert!(
        checked > 0,
        "no workflow names a path under src/, so this test proved nothing"
    );
}

/// [repo.layout]: the paths that decide whether a workflow RUNS. A filter that
/// matches nothing is the worst kind of stale reference — the workflow does not
/// fail, it silently stops triggering, and the site or the release simply stops
/// being rebuilt with nobody told.
#[test]
fn every_path_filter_a_workflow_triggers_on_matches_something() {
    let mut checked = 0_usize;
    for (name, workflow) in workflows() {
        let Some(events) = triggers(&workflow).and_then(Yaml::as_hash) else {
            continue;
        };
        for (event, body) in events {
            let event = event.as_str().unwrap_or("?");
            for key in ["paths", "paths-ignore"] {
                for pattern in body[key].as_vec().into_iter().flatten() {
                    let Some(pattern) = pattern.as_str() else {
                        continue;
                    };
                    // Everything before the first wildcard is literal, so it is
                    // the part that has to exist for the glob to match anything.
                    let literal = pattern
                        .split('*')
                        .next()
                        .unwrap_or("")
                        .trim_end_matches('/');
                    if literal.is_empty() {
                        continue;
                    }
                    assert!(
                        present(literal),
                        "{name} triggers on {event}.{key} `{pattern}`, and nothing in this tree \
                         is under {literal} — the workflow can never fire for it"
                    );
                    checked += 1;
                }
            }
        }
    }
    assert!(
        checked > 0,
        "no workflow filters on a path, so this test proved nothing"
    );
}

/// [release]: a job's `uses:` and its `working-directory`. The release calls
/// ci.yml so that "green" has one definition [release.verification], and
/// publishes the Dart package from a directory it names — both of which are
/// paths that moved once already.
#[test]
fn every_workflow_and_directory_a_job_names_exists() {
    let mut checked = 0_usize;
    for (name, workflow) in workflows() {
        for (job, body) in jobs(&workflow) {
            if let Some(uses) = body["uses"].as_str()
                && uses.starts_with("./")
            {
                assert!(
                    present(uses),
                    "{name} job `{job}` calls {uses}, which is not in this tree"
                );
                checked += 1;
            }

            let directories = body["defaults"]["run"]["working-directory"]
                .as_str()
                .into_iter()
                .chain(steps(body).filter_map(|step| step["working-directory"].as_str()));
            for directory in directories {
                assert!(
                    resolve(directory).is_dir(),
                    "{name} job `{job}` runs in {directory}, which is not a directory here"
                );
                checked += 1;
            }
        }
    }
    assert!(
        checked > 0,
        "no job names a workflow or a directory, so this test proved nothing"
    );
}
