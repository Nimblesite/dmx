//! Unit tests for the watcher's event classification [execution.modes].
//!
//! A separate file, and the only module here that is: watch.rs is close to
//! the 500-line ceiling every file in this repo is held to, and a test module
//! inline would put it over.

use super::*;
/// A directory of this test's own, emptied first so one run never inherits
/// the last one's leftovers.
fn scratch(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("dmx-watch-{name}"));
    drop(std::fs::remove_dir_all(&path));
    std::fs::create_dir_all(&path).expect("create scratch directory");
    path.canonicalize().expect("canonicalize scratch directory")
}

/// [execution.modes]: a tree is read when the burst settles, never when its
/// event arrives. This is the whole of the fix — a source written into a
/// new directory faster than the watcher could register for it is still
/// found, because the read happens after the write rather than racing it.
#[test]
fn a_tree_is_read_when_the_batch_resolves_not_when_it_is_claimed() {
    let root = scratch("resolve-after-claim");
    let nested = root.join("models");
    std::fs::create_dir_all(&nested).expect("create nested directory");

    let batch = claim(&nested, &[Scope::Directory(root)]);
    assert_eq!(batch.trees, BTreeSet::from([nested.clone()]));

    // The write the real race loses: it lands AFTER the event was claimed.
    let source = nested.join("user.dart");
    std::fs::write(&source, "class User {}").expect("write source");

    assert_eq!(
        batch.resolve().expect("resolve batch"),
        BTreeSet::from([source]),
        "a tree read on arrival would have found an empty directory"
    );
}

/// [execution.modes]: a Dart source stands for itself, never for its tree.
#[test]
fn a_dart_source_is_claimed_as_itself() {
    let root = scratch("source-claim");
    let source = root.join("user.dart");
    std::fs::write(&source, "class User {}").expect("write source");

    let batch = claim(&source, &[Scope::Directory(root)]);
    assert_eq!(batch.sources, BTreeSet::from([source]));
    assert!(batch.trees.is_empty(), "a file is not a tree");
}

/// [execution.modes]: a source missing when its event arrives, and back by the
/// time the burst settles, is the rename it always was. Every save is one —
/// the editor's, and dmx's own atomic write — so answering on arrival would
/// re-read the whole directory every time anybody pressed save.
#[test]
fn a_source_that_comes_back_within_the_burst_was_never_deleted() {
    let root = scratch("rename-window");
    let source = root.join("user.dart");

    let batch = claim(&source, &[Scope::Directory(root.clone())]);
    assert_eq!(
        batch.vanished,
        BTreeMap::from([(source.clone(), root)]),
        "a missing source is held, not answered"
    );

    // The other half of the rename, landing before the window closes.
    std::fs::write(&source, "class User {}").expect("write source");

    let resolved = batch.resolve().expect("resolve batch");
    assert_eq!(resolved, BTreeSet::from([source]));
}

/// [execution.modes] + [dartmacros.files]: one still missing when the window
/// closes was deleted, and its watched directory is re-read — which is what
/// re-runs the seed that writes a deleted generated file again.
#[test]
fn a_source_still_missing_when_the_burst_settles_re_reads_its_directory() {
    let root = scratch("deleted-source");
    let seed = root.join("schema.dart");
    std::fs::write(&seed, "class Schema {}").expect("write seed");

    let batch = claim(&root.join("customer_row.dart"), &[Scope::Directory(root)]);

    assert_eq!(
        batch.resolve().expect("resolve batch"),
        BTreeSet::from([seed]),
        "the directory the deleted file was in holds the seed that writes it"
    );
}

/// [surface.zero-config]: hidden entries are excluded, so a hidden
/// directory appearing is not a tree anybody asked to have read.
#[test]
fn a_hidden_directory_is_not_claimed() {
    let root = scratch("hidden-claim");
    let hidden = root.join(".git");
    std::fs::create_dir_all(&hidden).expect("create hidden directory");

    assert!(claim(&hidden, &[Scope::Directory(root)]).is_empty());
}
