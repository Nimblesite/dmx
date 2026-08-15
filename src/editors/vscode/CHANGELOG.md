# Changelog

<!-- Entries land under `## Unreleased` as they are written. The tag decides the
     version, and the release renames this heading to it [release.version]. -->

## Unreleased

- New icon: the dmx mark the website, favicon and docs already use. The
  extension had been shipping an older mark that matched nothing else.

## 0.2.0

- First Marketplace release.
- Watches every Dart package in the folder you open, not just `lib`
  [editor.extension.paths]. Set `dmx.paths` to name them yourself.
- Also finds the binary in `src/dmx/target/{release,debug}`
  [editor.extension.binary].
- Resolves reported paths against the folder dmx ran in.
- Removed a debug probe that ran on activation.

## 0.1.0

First release.

- Ships the `dmx` binary for the host platform and starts `dmx watch` when a
  Dart workspace folder opens [editor.extension.autostart].
- Resolves the binary from `dmx.path`, the bundle, the workspace's own
  `target/{release,debug}` build, then `PATH` [editor.extension.binary]. A
  platform with no bundle of its own installs the universal build and uses
  `PATH`.
- Highlights the `//#region` divider dmx owns, and the dmx annotations that put
  it there — the ones that generate scoped apart from the ones that configure
  [editor.dart-highlighting]. A labelled `//#region Helpers` is yours and is
  left alone.
- Highlights `.mustache` templates as Dart with Mustache tags injected over it,
  including tags inside Dart string literals [editor.template-highlighting].
- Commands: build, restart watcher, stop watcher, show output.
- Declines to run in a folder you have not trusted: dmx runs a binary over the
  files you open, and `dmx.path` can name one inside them.
