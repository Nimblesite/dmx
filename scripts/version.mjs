// The tag is the version, and it is carried into everything that ships
// [release.version].
//
// Nothing in the tree names a release. Every package file carries the
// placeholder `0.0.0`, and a release stamps the version its tag names into its
// own checkout — so nobody bumps a number before tagging, no tag is refused for
// a file somebody forgot, and the number can only ever be the one on the tag.
//
// Nothing here commits and nothing here pushes. The stamped tree is a pure
// function of (tagged commit, tag): run `make version VERSION=1.2.3` on the
// tagged commit and you have byte for byte what the release published.
//
// This half owns the bundle: the VSIX manifest and both changelogs. The other
// two files a release stamps are owned by the toolchains that own their format
// — `src/dmx/Cargo.toml` is never rewritten at all (the crate takes its version
// from `DMX_VERSION` at compile time), and `pubspec.yaml` is stamped by
// `src/dart_packages/dmx/tool/stamp_version.dart`, which splices the range
// `package:yaml` reports. No structured file anywhere is edited by pattern.
//
// The versions READ here are read the same way: the crate's from `cargo
// metadata`, the Dart package's from `dart pub deps --json`, the manifest's
// from `JSON.parse`. Each format is parsed by the tool that owns it.

import { execFileSync, spawnSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const REPO = path.join(path.dirname(fileURLToPath(import.meta.url)), '..');
const MANIFEST = path.join(REPO, 'src/editors/vscode/package.json');
const CHANGELOG = path.join(REPO, 'src/editors/vscode/CHANGELOG.md');
const PACKAGE = path.join(REPO, 'src/dart_packages/dmx');
const PACKAGE_CHANGELOG = path.join(PACKAGE, 'CHANGELOG.md');

/// The version every package file carries when no release has stamped it.
/// Valid semver for cargo, pub and the marketplace alike, and unmistakably not
/// a release — which is exactly what an untagged tree is.
export const PLACEHOLDER = '0.0.0';

/// The heading a release renames. Entries land under it as they are written,
/// so the notes for a release exist before anybody knows its number — which is
/// the whole point of letting the tag decide.
export const UNRELEASED = '## Unreleased';

/// The marketplace accepts `major.minor.patch` and nothing else: no `v`, no
/// pre-release suffix, no build metadata. Rejecting them here is what keeps a
/// tag like `v1.2.3-rc1` from failing halfway through a publish.
export function isReleaseVersion(version) {
  return /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(version);
}

/// The version a `v`-prefixed tag names, or `null` when the ref is not one.
/// Accepts the bare tag and the full ref a workflow is handed.
export function versionFromTag(ref) {
  const tag = String(ref ?? '').replace(/^refs\/tags\//, '');
  const version = tag.startsWith('v') ? tag.slice(1) : tag;
  return isReleaseVersion(version) ? version : null;
}

/// The version `cargo metadata` reports for the `dmx` package itself, given the
/// parsed document. Workspace members other than `dmx` are somebody else's.
export function crateVersion(metadata, name = 'dmx') {
  const found = (metadata?.packages ?? []).find((pkg) => pkg.name === name);
  return found?.version ?? null;
}

/// The version `dart pub deps --json` reports for the package being published,
/// given the parsed document. The root entry is the package itself; every other
/// entry is a dependency that happens to be resolved alongside it.
export function packageVersion(deps) {
  const found = (deps?.packages ?? []).find((pkg) => pkg.kind === 'root');
  return found?.version ?? null;
}

/// `manifest` with its version set, and every other byte left where it was.
/// Two-space JSON with a trailing newline is what `npm` and `vsce` write, so
/// re-stamping an already-correct manifest is a no-op on disk.
export function stamped(manifest, version) {
  const parsed = JSON.parse(manifest);
  return `${JSON.stringify({ ...parsed, version }, null, 2)}\n`;
}

/// The notes sitting under the `## Unreleased` heading, or `null` when there is
/// no such heading. Empty string when the heading is there with nothing under
/// it — a distinction a tag depends on, because a release with no notes ships a
/// blank page to every user who checks what changed.
export function unreleasedNotes(changelog) {
  const lines = changelog.split('\n');
  const at = lines.findIndex((line) => line.trim() === UNRELEASED);
  if (at === -1) {
    return null;
  }
  const rest = lines.slice(at + 1);
  const next = rest.findIndex((line) => line.startsWith('#'));
  return (next === -1 ? rest : rest.slice(0, next)).join('\n').trim();
}

/// `changelog` with its `## Unreleased` heading renamed to `version`, so the
/// marketplace and pub.dev pages name the release they are showing.
///
/// A changelog already naming `version` is returned untouched: re-running a
/// release, or stamping a tree somebody already stamped, must not accumulate.
export function released(changelog, version) {
  if (version === PLACEHOLDER || unreleasedNotes(changelog) === null) {
    return changelog;
  }
  return changelog
    .split('\n')
    .map((line) => (line.trim() === UNRELEASED ? `## ${version}` : line))
    .join('\n');
}

/// Whether a heading names this version, which is what a stamped changelog
/// looks like from the outside.
export function changelogHas(changelog, version) {
  return changelog.split('\n').some((line) => /^#{1,3}\s/.test(line) && line.includes(version));
}

/// Every reason the tree is not the placeholder tree a release stamps, as
/// human-readable complaints. Empty means a tag can stamp it.
///
/// This is the check ordinary CI runs, so a file that drifted off the
/// placeholder fails on the pull request that did it rather than on a tag.
export function unstamped({ crate, manifest, pubspec, changelog, packageChangelog }) {
  const problems = [];
  const placeholders = [
    ['src/dmx/Cargo.toml', crate],
    ['src/editors/vscode/package.json', manifest],
    ['src/dart_packages/dmx/pubspec.yaml', pubspec],
  ];
  for (const [file, version] of placeholders) {
    if (version !== PLACEHOLDER) {
      problems.push(
        `${file} is ${version} — the tag is the version, so this file carries ${PLACEHOLDER}`,
      );
    }
  }
  const changelogs = [
    ['src/editors/vscode/CHANGELOG.md', changelog],
    ['src/dart_packages/dmx/CHANGELOG.md', packageChangelog],
  ];
  for (const [file, text] of changelogs) {
    if (unreleasedNotes(text) === null) {
      problems.push(`${file} has no \`${UNRELEASED}\` heading for a release to rename`);
    }
  }
  return problems;
}

/// Every reason `version` cannot be released from this tree, on top of the
/// placeholder check above. Empty means the tag can publish.
export function unreleasable({ version, changelog, packageChangelog }) {
  const problems = [];
  if (!isReleaseVersion(version)) {
    problems.push(`\`${version}\` is not a marketplace version (major.minor.patch)`);
  }
  if (version === PLACEHOLDER) {
    problems.push(`\`${PLACEHOLDER}\` is the placeholder, not a release — tag a real version`);
  }
  const changelogs = [
    ['src/editors/vscode/CHANGELOG.md', changelog],
    ['src/dart_packages/dmx/CHANGELOG.md', packageChangelog],
  ];
  for (const [file, text] of changelogs) {
    // pub.dev warns on a changelog that does not name the version being
    // published, and the marketplace simply shows whatever is there. An empty
    // section renamed to `## 1.2.3` is a release page that says nothing.
    if (unreleasedNotes(text) === '') {
      problems.push(`${file} has \`${UNRELEASED}\` but nothing under it — write the notes`);
    }
  }
  return problems;
}

/// The crate version, straight from cargo. The crate is self-contained at
/// `src/dmx`, so the manifest is named explicitly rather than discovered from
/// the working directory — the repository root carries no Cargo.toml.
function fromCargo() {
  const manifest = path.join(REPO, 'src/dmx/Cargo.toml');
  const args = ['metadata', '--format-version', '1', '--no-deps', '--manifest-path', manifest];
  const out = execFileSync('cargo', args, {
    cwd: REPO,
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
  });
  return crateVersion(JSON.parse(out));
}

/// The Dart package's version, straight from pub. `pub deps` reads a RESOLVED
/// package, so `make version-check` resolves it first rather than assuming a
/// developer happened to have done so — an unresolved package is reported as
/// exactly that, never as a version mismatch nobody can act on.
/// `spawnSync`, not `execFileSync`: a package that is not resolved yet must come
/// back as a status this function can explain, never as a thrown error.
function fromPubspec() {
  const result = spawnSync('dart', ['pub', 'deps', '--json'], {
    cwd: PACKAGE,
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
  });
  if (result.status !== 0) {
    return null;
  }
  return packageVersion(JSON.parse(result.stdout));
}

/// Writes `version` into the VSIX manifest and both changelogs. The pubspec and
/// the crate are stamped elsewhere — see the header.
function stamp(version) {
  if (!isReleaseVersion(version)) {
    process.stderr.write(`\`${version}\` is not a version (expected major.minor.patch)\n`);
    return 1;
  }
  const written = [];
  const files = [
    [MANIFEST, 'src/editors/vscode/package.json', stamped],
    [CHANGELOG, 'src/editors/vscode/CHANGELOG.md', released],
    [PACKAGE_CHANGELOG, 'src/dart_packages/dmx/CHANGELOG.md', released],
  ];
  for (const [file, name, rewrite] of files) {
    const before = readFileSync(file, 'utf8');
    const after = rewrite(before, version);
    if (after !== before) {
      writeFileSync(file, after);
      written.push(name);
    }
  }
  process.stdout.write(
    written.length === 0
      ? `already at ${version}: nothing to stamp\n`
      : `stamped ${version}: ${written.join(', ')}\n`,
  );
  return 0;
}

/// Reports `problems` under `heading` and turns them into an exit code.
function report(heading, problems) {
  for (const problem of problems) {
    process.stderr.write(`${heading}: ${problem}\n`);
  }
  return problems.length === 0 ? 0 : 1;
}

/// Every version this tree declares, or the exit code that explains why one of
/// them could not be read.
function declared() {
  const crate = fromCargo();
  if (crate === null) {
    process.stderr.write('cargo metadata does not report a `dmx` package\n');
    return null;
  }
  const pubspec = fromPubspec();
  if (pubspec === null) {
    process.stderr.write(
      'src/dart_packages/dmx is not resolved, so its version cannot be read — ' +
        'run `dart pub get` there, or `make version-check`, which does it for you\n',
    );
    return null;
  }
  return {
    crate,
    pubspec,
    manifest: JSON.parse(readFileSync(MANIFEST, 'utf8')).version,
    changelog: readFileSync(CHANGELOG, 'utf8'),
    packageChangelog: readFileSync(PACKAGE_CHANGELOG, 'utf8'),
  };
}

/// `--check` proves the tree is the placeholder tree a tag stamps.
/// `--tag <ref>` additionally proves that tag can publish it.
/// `--stamp <version>` writes the version in.
/// `--print [ref]` writes the version a ref names, and nothing else.
function main(argv) {
  const flag = (name) => {
    const at = argv.indexOf(name);
    return at === -1 ? null : (argv[at + 1] ?? '');
  };

  const toStamp = flag('--stamp');
  if (toStamp !== null) {
    return stamp(toStamp);
  }

  if (argv.includes('--print')) {
    const ref = flag('--print');
    if (ref === '') {
      process.stdout.write(`${PLACEHOLDER}\n`);
      return 0;
    }
    const version = versionFromTag(ref);
    if (version === null) {
      process.stderr.write(`\`${ref}\` is not a release tag (expected \`v1.2.3\`)\n`);
      return 1;
    }
    process.stdout.write(`${version}\n`);
    return 0;
  }

  const tree = declared();
  if (tree === null) {
    return 1;
  }

  const tag = flag('--tag');
  if (tag === null) {
    const code = report('version-check', unstamped(tree));
    if (code === 0) {
      process.stdout.write(`every package file carries the placeholder ${PLACEHOLDER}\n`);
    }
    return code;
  }

  const version = versionFromTag(tag);
  if (version === null) {
    process.stderr.write(`\`${tag}\` is not a release tag (expected \`v1.2.3\`)\n`);
    return 1;
  }
  const problems = [...unstamped(tree), ...unreleasable({ version, ...tree })];
  const code = report(`release ${version}`, problems);
  if (code === 0) {
    process.stdout.write(`release ${version}: this tree is ready to be stamped and published\n`);
  }
  return code;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  process.exitCode = main(process.argv.slice(2));
}
