// [release.version]: the checks and the stamp a release runs, driven with the
// tags and versions a release would hand them. The functions under test are the
// ones the tag path calls — not a re-implementation of them, which would only
// ever prove itself right.

import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { test } from 'node:test';
import path from 'node:path';

import {
  PLACEHOLDER,
  UNRELEASED,
  changelogHas,
  crateVersion,
  isReleaseVersion,
  packageVersion,
  released,
  stamped,
  unreleasable,
  unreleasedNotes,
  unstamped,
  versionFromTag,
} from './version.mjs';

const REPO = path.join(path.dirname(fileURLToPath(import.meta.url)), '..');
const read = (relative) => readFileSync(path.join(REPO, relative), 'utf8');

/// A tree in the shape every check expects: placeholders everywhere, both
/// changelogs holding notes nobody has named a version for yet.
const placeholderTree = () => ({
  crate: PLACEHOLDER,
  manifest: PLACEHOLDER,
  pubspec: PLACEHOLDER,
  changelog: `# Changelog\n\n${UNRELEASED}\n\n- Something.\n\n## 0.1.0\n\n- First.\n`,
  packageChangelog: `# Changelog\n\n${UNRELEASED}\n\n- Something else.\n`,
});

test('a release version is three numbers and nothing else', () => {
  assert.ok(isReleaseVersion('0.1.0'));
  assert.ok(isReleaseVersion('12.4.199'));
  // The marketplace rejects every one of these, and it is cheaper to find out
  // here than halfway through a publish.
  for (const bad of ['1.2', 'v1.2.3', '1.2.3-rc1', '1.2.3+build', '01.2.3', '', 'latest']) {
    assert.ok(!isReleaseVersion(bad), bad);
  }
});

test('a tag names its version, in either form a workflow hands it over', () => {
  assert.equal(versionFromTag('v1.2.3'), '1.2.3');
  assert.equal(versionFromTag('refs/tags/v1.2.3'), '1.2.3');
  assert.equal(versionFromTag('1.2.3'), '1.2.3');
});

test('a ref that is not a release tag names no version', () => {
  for (const ref of ['refs/heads/main', 'v1.2.3-rc1', 'nightly', '', null, undefined]) {
    assert.equal(versionFromTag(ref), null, String(ref));
  }
});

test('the crate version comes from the dmx package, not whichever came first', () => {
  const metadata = {
    packages: [
      { name: 'anyhow', version: '1.0.104' },
      { name: 'dmx', version: '4.5.6' },
    ],
  };
  assert.equal(crateVersion(metadata), '4.5.6');
  assert.equal(crateVersion(metadata, 'absent'), null);
  assert.equal(crateVersion({}), null);
});

/// `dart pub deps --json` lists the package being published alongside every
/// dependency resolved with it, and a dependency named `dmx` would not be the
/// one being released — the root marker is what distinguishes them.
test('the Dart package version comes from the root entry, not a dependency', () => {
  const deps = {
    root: 'dmx',
    packages: [
      { name: 'meta', version: '1.16.0', kind: 'direct' },
      { name: 'dmx', version: '7.8.9', kind: 'root' },
    ],
  };
  assert.equal(packageVersion(deps), '7.8.9');
  assert.equal(packageVersion({ packages: [{ name: 'dmx', version: '1.0.0', kind: 'direct' }] }), null);
  assert.equal(packageVersion({}), null);
});

test('stamping sets the version and disturbs nothing else', () => {
  const before = JSON.parse(read('src/editors/vscode/package.json'));
  const after = JSON.parse(stamped(read('src/editors/vscode/package.json'), '9.9.9'));
  assert.equal(after.version, '9.9.9');
  assert.deepEqual({ ...after, version: null }, { ...before, version: null });
});

/// If re-stamping the shipped manifest with the version it already has changed
/// a byte, every local `make vsix` would dirty the tree.
test('stamping a manifest with the version it already has is a no-op', () => {
  const manifest = read('src/editors/vscode/package.json');
  assert.equal(stamped(manifest, JSON.parse(manifest).version), manifest);
});

test('a changelog section counts only when a heading names the version', () => {
  const changelog = '# Changelog\n\n## 0.2.0\n\n- Something.\n\n## 0.1.0\n\n- First.\n';
  assert.ok(changelogHas(changelog, '0.2.0'));
  assert.ok(changelogHas(changelog, '0.1.0'));
  assert.ok(!changelogHas(changelog, '0.3.0'));
  // A version mentioned in prose is not a section about it.
  assert.ok(!changelogHas('# Changelog\n\nFixed since 0.4.0.\n', '0.4.0'));
});

test('the unreleased notes are the lines under the heading, and stop at the next one', () => {
  const changelog = `# Changelog\n\n${UNRELEASED}\n\n- Added a thing.\n\n## 0.1.0\n\n- First.\n`;
  assert.equal(unreleasedNotes(changelog), '- Added a thing.');
});

/// The distinction the tag gate turns on: no heading at all is a changelog
/// nobody set up, and a heading with nothing under it is a release whose notes
/// nobody wrote. Both block a tag, and they say different things.
test('a missing heading and an empty one are told apart', () => {
  assert.equal(unreleasedNotes('# Changelog\n\n## 0.1.0\n\n- First.\n'), null);
  assert.equal(unreleasedNotes(`# Changelog\n\n${UNRELEASED}\n\n## 0.1.0\n\n- First.\n`), '');
  assert.equal(unreleasedNotes(`# Changelog\n\n${UNRELEASED}\n`), '');
});

test('releasing renames the heading and touches nothing else', () => {
  const changelog = `# Changelog\n\n${UNRELEASED}\n\n- Added a thing.\n\n## 0.1.0\n\n- First.\n`;
  assert.equal(
    released(changelog, '1.2.3'),
    '# Changelog\n\n## 1.2.3\n\n- Added a thing.\n\n## 0.1.0\n\n- First.\n',
  );
  assert.ok(changelogHas(released(changelog, '1.2.3'), '1.2.3'));
});

/// `make vsix` on a laptop stamps the placeholder, and a target that rewrote
/// the changelog every time it ran would leave every developer with a dirty
/// tree they did not ask for.
test('stamping the placeholder leaves the changelog alone', () => {
  const changelog = `# Changelog\n\n${UNRELEASED}\n\n- Added a thing.\n`;
  assert.equal(released(changelog, PLACEHOLDER), changelog);
});

/// A release that has to be re-run — a marketplace outage, a cancelled job —
/// stamps a tree that may already be stamped.
test('releasing an already-released changelog is a no-op', () => {
  const changelog = `# Changelog\n\n${UNRELEASED}\n\n- Added a thing.\n`;
  const once = released(changelog, '1.2.3');
  assert.equal(released(once, '1.2.3'), once);
});

test('a placeholder tree has nothing to complain about', () => {
  assert.deepEqual(unstamped(placeholderTree()), []);
});

test('every file that drifted off the placeholder is reported at once', () => {
  const problems = unstamped({
    ...placeholderTree(),
    crate: '1.2.2',
    manifest: '0.1.0',
    pubspec: '0.9.0',
    changelog: '# Changelog\n\n## 0.1.0\n',
    packageChangelog: '# Changelog\n\n## 0.1.0\n',
  });
  assert.equal(problems.length, 5);
  assert.ok(problems.some((p) => p.includes('Cargo.toml is 1.2.2')));
  assert.ok(problems.some((p) => p.includes('package.json is 0.1.0')));
  assert.ok(problems.some((p) => p.includes('pubspec.yaml is 0.9.0')));
  assert.ok(problems.some((p) => p.includes('src/editors/vscode/CHANGELOG.md')));
  assert.ok(problems.some((p) => p.includes('src/dart_packages/dmx/CHANGELOG.md')));
});

/// pub.dev publishes the version the pubspec names and a pub release can only
/// be retracted, never replaced. A pubspec carrying a real version is a file
/// somebody edited by hand, and it has to stop CI on its own [release.version].
test('a pubspec left off the placeholder stops CI by itself', () => {
  const problems = unstamped({ ...placeholderTree(), pubspec: '1.2.2' });
  assert.equal(problems.length, 1);
  assert.ok(problems[0].includes('pubspec.yaml is 1.2.2'));
});

test('a tag with notes behind it is releasable', () => {
  assert.deepEqual(unreleasable({ version: '1.2.3', ...placeholderTree() }), []);
});

test('a tag that is not a marketplace version is refused before anything ships', () => {
  const problems = unreleasable({ version: '1.2.3-rc1', ...placeholderTree() });
  assert.equal(problems.length, 1);
  assert.ok(problems[0].includes('not a marketplace version'));
});

/// `v0.0.0` is the placeholder, so tagging it would publish a release that
/// every untagged build in existence also claims to be.
test('the placeholder cannot be tagged', () => {
  const problems = unreleasable({ version: PLACEHOLDER, ...placeholderTree() });
  assert.ok(problems.some((p) => p.includes('is the placeholder')));
});

test('a release with no notes written for it is refused', () => {
  const tree = placeholderTree();
  const problems = unreleasable({
    version: '1.2.3',
    ...tree,
    packageChangelog: `# Changelog\n\n${UNRELEASED}\n\n## 0.1.0\n`,
  });
  assert.equal(problems.length, 1);
  assert.ok(problems[0].includes('src/dart_packages/dmx/CHANGELOG.md'));
  assert.ok(problems[0].includes('write the notes'));
});

/// [release.version]: the release workflow captures `--print` into a step
/// output rather than deriving the version itself, so this is release machinery
/// and is driven the way the release drives it. What a workflow captures has to
/// be the version and nothing else: a second line, a log message or a stray
/// space all become part of `needs.preflight.outputs.version`, which every
/// later job stamps, builds and publishes with.
test('--print writes the version a tag names, alone', () => {
  const printed = execFileSync(
    process.execPath,
    [path.join(REPO, 'scripts/version.mjs'), '--print', 'v1.2.3'],
    { cwd: REPO, encoding: 'utf8' },
  );
  assert.equal(printed, '1.2.3\n');
});

/// The rehearsal has no tag. It must still produce a version for the matrix to
/// build with, and the honest one is the placeholder — a dispatch is not a
/// release, and a bundle it produced must not claim to be one.
test('--print with no tag writes the placeholder', () => {
  const printed = execFileSync(
    process.execPath,
    [path.join(REPO, 'scripts/version.mjs'), '--print'],
    { cwd: REPO, encoding: 'utf8' },
  );
  assert.equal(printed, `${PLACEHOLDER}\n`);
});

/// The gate that matters: what is committed right now must be stampable. A tree
/// that cannot pass its own placeholder check is a tree whose next tag fails in
/// CI instead of here. The crate and the pubspec are read by `--check` through
/// cargo and pub; the two files this test can read without a toolchain are read
/// here.
test('the committed tree carries the placeholder and an Unreleased heading', () => {
  assert.equal(JSON.parse(read('src/editors/vscode/package.json')).version, PLACEHOLDER);
  assert.notEqual(unreleasedNotes(read('src/editors/vscode/CHANGELOG.md')), null);
  assert.notEqual(unreleasedNotes(read('src/dart_packages/dmx/CHANGELOG.md')), null);
});
