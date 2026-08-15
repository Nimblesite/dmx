# dmx — Emission and Backends

Part of the [dmx specification](SPEC.md).

## [emission] Emission and Backends

### [emission.backends] Backends

| Backend | Status | Placement | Consumer boilerplate |
|---|---|---|---|
| **`inline`** | **REQUIRED, default** | Inside the class body, before `}` | One region comment (auto-inserted) |
| `part` | REQUIRED | Sibling `.g.dart` | `part` directive + `with _$X` + delegating factory |
| `augment` | OPTIONAL, flagged | `augment class C { … }` | `part` directive |

All three consume the same fragments; only placement differs. That is what makes G9 cheap.

### [emission.part-backend] `part` backend

| Role | Mechanism | Boilerplate |
|---|---|---|
| `mixin` | `mixin _$User` with abstract getters; can override `==`, `hashCode`, `toString` | `class User with _$User` |
| `extension` | `extension $UserCopyWith on User`; cannot override existing members | None |
| `topLevel` | `_$UserFromJson` / `_$UserToJson` | A delegating factory |
| `member` | **Rejected** — `DMX6001`; part files cannot inject members into an existing class | — |

Recommended when generated code must be gitignored ([emission.inline-backend.lints]).

### [emission.inline-backend] `inline` backend

#### [emission.inline-backend.form] Form

Generated members are written as the last content inside the class body, between markers, before the closing brace:

```dart
@dmx('model')
class User {
  const User({required this.id, this.email});
  final String id;
  final String? email;

  //#region dmx:generated builtin/model@1.0.0 b3:9f2c4ae1d38b7c05 — DO NOT EDIT
  factory User.fromJson(Map<String, dynamic> _$j4n2_json) => User(
        id: _$j4n2_json['id'] as String,
        email: _$j4n2_json['email'] as String?,
      );

  Map<String, dynamic> toJson() => <String, dynamic>{
        'id': id,
        'email': email,
      };

  @override // ignore: unnecessary_overrides
  bool operator ==(Object _$k3m9_other) =>
      identical(this, _$k3m9_other) ||
      (_$k3m9_other is User && _$k3m9_other.id == id && _$k3m9_other.email == email);

  @override
  int get hashCode => Object.hash(runtimeType, id, email);

  static const Object? _$unset = Object();

  User copyWith({String? id, Object? email = _$unset}) => User(
        id: id ?? this.id,
        email: identical(email, _$unset) ? this.email : email as String?,
      );
  //#endregion dmx:generated
}
```

`//#region` folds by default in VS Code and IntelliJ, collapsing the block to a single line.

Because members land directly in the class, `inline` needs **no** `part` directive, **no** mixin, **no** abstract getter redeclaration, and **no** delegating factory. All fragment roles collapse to `member`.

#### [emission.inline-backend.region-location] Region location (normative)

The driver MUST locate regions via **CST comment tokens**, never by regex or line scanning. A `//#region dmx:generated` sequence appearing inside a string literal, a doc comment, or a nested class MUST NOT be matched.

1. Resolve the target class's body span via `Frontend::class_body_span`.
2. Enumerate comment tokens within that span at the class body's direct nesting depth.
3. A region is a `//#region dmx:generated` token and the nearest following `//#endregion dmx:generated` token at the same depth.
4. Unmatched start, unmatched end, overlapping regions, or a region not directly within the class body MUST produce `DMX6102`.
5. Exactly one region per class per macro. Multiple macros produce multiple regions, ordered lexicographically by macro id.

#### [emission.inline-backend.insertion] Insertion

If no region exists, the driver MUST insert one immediately before the class body's closing brace, preserving the file's existing indentation style and line endings, and separated from the preceding member by exactly one blank line.

Insertion is a modification of a user file and therefore MUST be gated: `dmx fix`, or `dmx build --insert-regions`. It MUST NOT happen implicitly during a default `build`. A missing region under default `build` produces `DMX6002` with the exact suggested edit.

#### [emission.inline-backend.byte-exactness] Byte-exactness outside the region

Every byte outside the region markers MUST be preserved unchanged. The driver MUST verify this by comparing the pre- and post-write file with the region excised; mismatch MUST abort before writing and produce `DMX6103`. This is the invariant that makes writing into user files safe.

#### [emission.inline-backend.region-recovery] Region recovery

There is no hash in the divider and no tamper check. Generation is a pure function of the source, so a region that differs from what dmx would emit is simply out of date, and `--check` ([execution]) reports that as drift. Replacing it is never a clobber: the bytes between the dividers are machine-owned by definition ([emission.inline-backend.region-location]).

A human editing generated code therefore needs no special error — but it does need recovery, because deleting generated members usually leaves the file unparseable. An orphaned `};` closes the class body early, and the parser then reports the class's *real* closing brace as an error **outside** the region, so no containment test can attribute the damage to dmx.

The driver MUST therefore, when the input fails validation ([validation]):

1. Empty every machine-owned region, leaving its dividers in place.
2. Re-parse the result.

- **Parses** → everything broken was inside a divider. Regenerate normally; that *is* the repair.
- **Still fails** → the damage is the author's own. MUST report the original `DMX4001` and MUST NOT write.

Recovery MUST converge on the same bytes as generating from an already-valid file, so a repaired file is a fixed point and a watcher cannot loop writing to itself.

#### [emission.inline-backend.no-op-writes] No-op writes

If the newly rendered region body is byte-identical to the existing one, the driver MUST NOT write the file at all — mtime MUST be preserved. This prevents watch-mode feedback loops, spurious IDE reloads, and rebuild storms.

#### [emission.inline-backend.concurrent-modification] Concurrent modification

Between read and write the file may change (an editor autosave). The driver MUST perform a compare-and-swap: re-read and re-verify the source hash immediately before rename; on mismatch, abort that target with `DMX6101` and, under `watch`, retry once after the debounce interval.

#### [emission.inline-backend.lints] Lints and coverage

- The driver MUST NOT emit `// ignore_for_file:` under `inline` — it would suppress lints across the user's own code. Suppressions MUST be per-line `// ignore:` comments attached to generated lines only.
- The driver SHOULD wrap the region in `// coverage:ignore-start` / `// coverage:ignore-end` where the project's coverage tooling honours them, since generated lines otherwise count against the user's source file with no way to exclude them.
- Generated members SHOULD carry a marker doc comment or `@pragma` sufficient for future tooling to identify them; the region markers remain authoritative.

#### [emission.inline-backend.version-control] Version control

`inline` output cannot be gitignored — it lives in real source files. Two supported strategies:

**A. Commit the regions (default).** Simple, works everywhere, no setup. Cost: every field addition produces a diff several times larger than the edit. Mitigate with `.gitattributes`:

```
*.dart diff=dart
```

and a `linguist-generated` marker where the host supports region-level attribution (most do not — this remains a real cost).

**B. Git clean/smudge filter (zero noise).** The clean filter strips region bodies on stage; the smudge filter is a no-op; `dmx build` restores content after checkout. The committed blob contains an empty region; the working tree contains generated code.

```
# .gitattributes
*.dart filter=dmx
```
```
git config filter.dmx.clean "dmx git-filter clean"
git config filter.dmx.smudge cat
```

The driver MUST provide `dmx git-filter clean`, which reads a Dart file on stdin and writes it with all `dmx:generated` region bodies emptied (markers and header retained, hash zeroed), and MUST guarantee it is byte-exact outside regions ([emission.inline-backend.byte-exactness]).

Cost, stated plainly: per-clone `git config` (filters are not configured by `.gitattributes` alone, by design), a CI step running `dmx build` after checkout, and diffs that appear empty for generated code in review. Strategy B is opt-in; Strategy A is the default because it requires nothing.

**C. Use the `part` backend instead** if gitignored generated code matters more than removing boilerplate. This is a legitimate choice and MUST remain fully supported.

#### [emission.inline-backend.non-class-targets] Non-class targets

Top-level helpers that cannot live inside a class body MAY be emitted into a **file-level region** placed after the last top-level declaration, with the same markers, hashing, and byte-exactness rules. The context builder SHOULD prefer in-class `static const` members over top-level declarations precisely to avoid needing this — `static const Object? _$unset = Object();` replaces a top-level sentinel class entirely.

### [emission.augment-backend] `augment` backend

Gated behind `--enable-augmentations`; MUST warn that Dart augmentations are not stable. Consumes the same `member` fragments as `inline`, so the two are near-interchangeable and migration is a flag flip.

### [emission.header-formatting] Header and formatting

Region and file headers carry tool version, source, target, macro id, and content hash. Output SHOULD be emitted already close to `dart format`; the deterministic built-in normalizer is the default and shelling out to `dart format` MUST be opt-in, as it adds a Dart process dependency and would negate G5.

Under `inline`, the normalizer MUST match the surrounding file's indentation width and line endings rather than imposing its own.

---

