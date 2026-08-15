# dmx — Extension Layers and Dart Macros

Part of the [dmx specification](SPEC.md).

## [extensions] Extension Layers

Absent by default. Nothing in §[surface]–5 depends on this section.

### [extensions.points] Three extension points

| Point | Signature | Purpose |
|---|---|---|
| **Macro** | `Invocation → Fragment` | Define a new `@dmx('name')` — normative surface in [dartmacros] |
| **Transform** | `Context → Context` | Add, derive, or rewrite context variables before rendering |
| **Engine** | `(Context, Template) → Text` | Replace the renderer entirely |

All three may be implemented in **any language** via the worker protocol ([extensions.worker-protocol]); transforms and engines additionally in Rust in-process. A Dart implementation is simply one worker; so is Python, TypeScript, Go, or another Rust binary. The macro point has a privileged, zero-config Dart path specified in [dartmacros] — that section is the consumer-facing story, this section is the wire truth beneath it.

An engine is how "ad hoc Dart inside templates" is achieved: a Dart worker that receives the context and a template and returns text is free to interpret that template however it likes, including as executable Dart.

### [extensions.obligations] Obligations (normative)

Any transform or engine MUST be:

1. **Pure** — output depends only on declared inputs. No clock, network, or ambient environment.
2. **Deterministic** — including map iteration order.
3. **Bounded** — subject to the driver's step, memory, and wall-clock limits.

`dmx` cannot enforce purity in a foreign process. It therefore:

- Includes the worker's binary/script content hash in the cache key ([execution.cache-key]).
- Provides `--verify-extensions`, which runs every transform and engine twice per target and fails with `DMX7003` on any difference. This SHOULD run in CI.
- Executes workers with a scrubbed environment and, where the OS supports it, a restricted working directory.

### [extensions.worker-protocol] Worker protocol

Newline-delimited JSON over stdin/stdout. One long-lived process per worker, reused across all targets in a build — process startup is amortized once, not per target.

**Handshake** (driver → worker on spawn):
```json
{"v":1,"op":"hello"}
```
```json
{"v":1,"name":"dart-transform","version":"1.0.0","contextVersion":1,"ops":["transform"]}
```

**Transform:**
```json
{"v":1,"op":"transform","id":"t1","context":{…},"options":{…}}
```
```json
{"v":1,"id":"t1","context":{…},"diagnostics":[]}
```

**Render:**
```json
{"v":1,"op":"render","id":"r7","context":{…},"template":"…","name":"equality.mustache"}
```
```json
{"v":1,"id":"r7","text":"…","spans":[…],"introduced":["other","json"],"diagnostics":[]}
```

Requirements:

- Requests carry an `id`; responses MUST echo it. Workers MAY process concurrently and respond out of order.
- `introduced` is REQUIRED on render responses and lists every identifier the output *binds*. It drives [hygiene]. Omitting an introduced binding is a hygiene defect, detected by §9.5.
- `spans` maps output byte offsets to template line/column; REQUIRED for [diagnostics] diagnostics.
- `contextVersion` mismatch MUST produce `DMX7001` and abort. This is a wire version between programs, not a user-authored artifact.
- Stderr is captured and surfaced under `--verbose`. A worker writing to stdout outside the protocol MUST produce `DMX7002`.
- Crash, timeout, or malformed frame MUST fail the build, never silently skip a target.

### [extensions.performance-tiers] Performance tiers

| Mechanism | Per-target cost | Startup | Verdict |
|---|---|---|---|
| Rust, in-process | ~1–50 µs | none | Built-ins |
| WASM component, in-process | ~50–500 µs | ~1 ms | Good |
| **AOT-compiled worker** (`dart compile exe`, Go, Rust) | ~50–200 µs + IPC | ~10 ms once | **Recommended for foreign languages** |
| JIT worker (`dart run`, `node`, `python`) | ~100 µs–1 ms + IPC | ~150–400 ms once | Acceptable; amortized |
| Subprocess per target | ~150 ms × N | per target | **MUST NOT** — violates [performance] |

Spawning a process per target is prohibited. The persistent-worker model is what keeps "any language" compatible with the performance budget.

### [extensions.dart-language] Dart as the extension language

The recommended path is a Dart worker AOT-compiled with `dart compile exe`, giving ~10 ms one-time startup and microsecond-scale per-target handling. `dart run` is acceptable during development.

This makes Dart a first-class extension language without a Dart runtime on the default path: consumers using built-in macros never spawn a Dart process, and never install one for `dmx`'s sake.

An implementation MAY additionally offer an embedded Dart-subset interpreter in Rust for zero-IPC scripting. This is explicitly **not** required and MUST NOT be a prerequisite for [extensions] shipping — the worker protocol is the general mechanism, and an embedded interpreter is a later optimization for one language.

### [extensions.declaration] Declaration

Extensions are declared in `dmx.yaml`, which remains optional — a project without extensions has no config file:

```yaml
extensions:
  transform:
    - command: ["tool/dmx/enrich"]
      applyTo: ["Model"]
  engine:
    dart-templates:
      command: ["tool/dmx/dart_engine"]
      extensions: [".dart.tmpl"]
```

Macro workers are the exception to this section's declaration requirement: the Dart macro worker is discovered by convention ([dartmacros.discovery]) and MUST NOT require a `dmx.yaml` entry. Non-Dart macro workers MAY be declared here under a `macros:` key with the same `command` shape.

---

## [dartmacros] User-Defined Macros in Dart

**The catalogue is open.** A project that wants `@dmx('audit')` writes it in Dart — in its own repository, against a typed API, with no fork, no Rust, and no configuration. At the use site a user-defined macro is indistinguishable from a built-in: the same `@dmx('name')` trigger ([surface.annotations]), the same region, the same guarantees. This section is normative; [extensions] is the substrate it stands on.

### [dartmacros.discovery] Discovery (zero config)

`tool/dmx/macros.dart`, found **from the source being generated**, is that package's **macro worker**:

- `dmx` MUST discover it by path convention alone. No `dmx.yaml`, no registration, no pubspec marker. A project without the file has no macro worker and pays nothing — the default path never spawns a Dart process ([extensions.dart-language]).
- The lookup MUST start at the directory holding the file being generated and walk upwards, taking the first `tool/dmx/macros.dart` it finds; the working directory is the fallback, used only when that walk finds none. A file therefore generates identically wherever `dmx` was launched from. Anchoring the lookup to the working directory instead is prohibited: an editor starts the watcher at the workspace root and `make` runs it at a repo root, and in a tree holding more than one package neither is the package root — so every user `@dmx` in it would silently stay inert, and the files those macros own would be read as no longer produced and collected ([dartmacros.files]).
- One worker process per worker file, spawned once and reused for every target ([extensions.performance-tiers]). Per-target spawning remains prohibited. A pass covering several packages MUST run one worker per package and MUST NOT offer one package's annotations to another's worker.
- `dart run tool/dmx/macros.dart` MUST work during development. `dmx` SHOULD transparently maintain an AOT compile (`dart compile exe`) keyed by the worker's source content hash under `.dart_tool/dmx/`, so steady-state builds pay ~10 ms once, not JIT startup per session.

### [dartmacros.api] The authoring surface

A dedicated library, `package:dmx/macros.dart`, separate from `package:dmx/dmx.dart` so consumers never depend on the protocol. Both ship in the one `dmx` package — a project adds a single dependency — but the library boundary is normative: `package:dmx/dmx.dart` MUST NOT import the authoring library, so an application that only annotates its models never pulls in the protocol, its `dart:io` use, or anything that would cost it web compatibility. Only a macro worker imports `package:dmx/macros.dart`. The whole surface:

```dart
import 'package:dmx/macros.dart';

final class Audit extends DmxMacro {
  @override
  String get name => 'audit';

  @override
  DmxOutput expand(DmxInvocation invocation) {
    final fields = invocation.declaration.fields
        .map((f) => "'${f.name}': ${f.name}")
        .join(',\n      ');
    return DmxFragment('''
  Map<String, Object?> get auditEntry => {
      $fields,
    };
''');
  }
}

void main() => dmxServeMacros([Audit()]);
```

`DmxInvocation` is the complete, typed view of what the front end resolved:

| Member | Meaning |
|---|---|
| `declaration` | The annotated declaration: `name`, `kind` (class/enum), `modifiers`, `typeParams`, `extendsName`, `interfaces`, `fields[]`, `values[]`, `methods[]`, `docComment` |
| `declaration.fields[]` | `name`, `type` (name, nullability, type arguments, collection-ness), `annotations[]` with raw-source args ([surface.annotations]) |
| `args` | The `@dmx('name', {…})` map, keys to raw CST source, unevaluated |
| `file` | Sibling declarations, for relational macros ([frontend.name-index]) — a union reads its variants, a fake reads its interface |
| `backend`, `memberIndent` | Emission facts the fragment must respect ([context.root]) |

`DmxOutput` is a sealed type with two cases, so an author's `switch` over a result is exhaustive: `DmxFragment(text, {introduced, files})` or `DmxRefusal(code, message)`. A fragment fills the annotated declaration's region; its `files` additionally author whole sibling files the macro names ([dartmacros.files]). Refusal is a value, not an exception — macro authors surface author-facing diagnostics the same way built-ins do ([diagnostics]). A macro that throws crashes the worker, and a crashed worker MUST fail the build ([extensions.worker-protocol]) — never silently skip a target.

`dmx explain <file>` MUST print the exact `DmxInvocation` JSON a declaration produces, so a macro author sees their input before writing a line.

### [dartmacros.protocol] The `expand` op

Macro workers speak the worker protocol ([extensions.worker-protocol]) with one additional op. The handshake response lists the macro names the worker serves:

```json
{"v":1,"name":"macros","version":"1.0.0","contextVersion":1,"ops":["expand"],"macros":["audit"]}
```

```json
{"v":1,"op":"expand","id":"e1","macro":"audit","invocation":{…}}
```
```json
{"v":1,"id":"e1","text":"…","introduced":["auditEntry"],"diagnostics":[]}
```

`introduced` and `spans` carry the same obligations as render responses. A reply MAY additionally carry `files`, whole sibling files the macro authors ([dartmacros.files]). `dmxServeMacros` implements all of this; a macro author never sees a frame.

### [dartmacros.render] The `render` op, reversed

**A user macro MUST be able to render a Mustache template with dmx's own engine.** Custom macros and templates are one system, not alternatives: a macro computes what no template could work out for itself, and a template lays out what no string-building should. Forcing a macro author to choose between them means either hand-built Dart with no editable layout, or a template with no way to answer a question about the project.

The `render` op already exists in the other direction, where dmx asks a worker to render ([extensions.worker-protocol]). This is the same frame, sent the other way:

```json
{"v":1,"op":"render","id":"r1","name":"model.mustache","template":"…","context":{…}}
```
```json
{"v":1,"id":"r1","text":"…"}
```

Normatively:

- **Both directions on one connection.** While a driver is awaiting an `expand` reply, a frame arriving with an `op` is a request *from* the worker and MUST be answered in place; the first frame without one is the reply to the outstanding request. Ids distinguish the two conversations. A driver that read every frame as a reply would deadlock any macro that renders.
- **The same engine.** The driver MUST render with the engine, the standalone-tag handling ([rendering]), and the whitespace normalizer the built-in catalogue uses. A macro's output and a built-in's therefore obey identical whitespace law, and a project's template dialect is the dialect dmx documents — not a second one shipped by whichever Mustache package the macro author reached for.
- **The context is JSON.** A macro has no Rust `Content` struct, so the model arrives as ordinary JSON and MUST be read with Mustache truthiness: `null`, `false`, `0`, `""`, `[]`, and `{}` are falsy. A variable tag naming a list or an object MUST render nothing rather than a serialization of it. Name resolution MUST walk out to enclosing contexts, so a section still sees the root model.
- **Failure is a value.** A template that does not compile MUST be answered with an `error` frame, not by failing the build at that point. The macro asked a question; it gets an answer it can return as a `DmxRefusal` in its author's terms ([dartmacros.api], [diagnostics]). Failing the build here instead strands the worker on a reply that never arrives. `DMX7009` covers a malformed request and an uncompilable template.
- **The macro owns the template.** dmx supplies the engine, not the template directory: the worker sends the template *source*, so where a project keeps its templates, and whether it lets a user override them, is the project's design rather than a convention dmx imposes.
- **`expand` may be asynchronous.** An expansion that awaits a render is still one expansion. Replies MUST be emitted in the order the requests arrived, so generation stays deterministic ([extensions.obligations]).

The authoring surface is `invocation.templates.render(source, context, name:)`, returning a `Result<String, DmxRefusal>` ([dartmacros.api]). A macro that builds its Dart directly never calls it and pays nothing.

### [dartmacros.resolution] Resolution

- Built-in names win. A worker declaring a name the binary already registers MUST fail the handshake with `DMX7005` — built-ins are never silently shadowed, so upgrading `dmx` can never quietly change whose code generates.
- Two user macros declaring the same name MUST fail with `DMX7006`.
- A `@dmx('name')` matching neither a built-in nor a declared user macro MUST produce warning `DMX2013` naming the nearest known macro. With an open catalogue, a typo can no longer be silently inert.

### [dartmacros.pipeline] Same pipeline, same bar

A user fragment enters the pipeline where a built-in template's output does: normalize → hygiene ([hygiene]) → validation ([validation]) → emission ([emission]). Nothing is exempt:

- The fragment MUST satisfy every generated-Dart rule and pass `dart analyze --fatal-infos` (C6). A user macro that emits a `throw` or a cast fails validation exactly as a built-in would.
- Region headers carry the macro id as `user/<name>@<worker version>` ([emission.header-formatting]), so a region names who generated it.
- Purity, determinism, and boundedness obligations apply ([extensions.obligations]); the worker's content hash is in the cache key ([execution.cache-key]); `--verify-extensions` covers macro workers.
- Several macros on one declaration — built-in or user, mixed freely — emit ordered fragments into the one region ([rendering]).

### [dartmacros.files] Macro-authored files

A region can only hold members of the class the author already named — which means the author is still typing one class, one file, and one name per generated thing. When the source of truth enumerates its own shape (a database's tables, an API's endpoints), the macro MUST be able to author **whole files**, named by the macro, one annotation for all of them.

An `expand` reply MAY carry `files` alongside `text`:

```json
{"v":1,"id":"e1","text":"…","introduced":["tables"],
 "files":[{"name":"customer_row.dart","text":"…"},
          {"name":"order_row.dart","text":"…"}]}
```

The annotated declaration is the **seed**: its fragment fills its region as ever, and each entry in `files` becomes a complete Dart file beside the seed's own file. Normatively:

- **Naming.** `name` MUST be a bare file name ending in `.dart` — no path separator, no leading dot, a non-empty stem. Anything else is `DMX7007`. The macro controls the name; the seed's directory anchors where it lands.
- **Ownership marker.** The driver — never the macro — prepends line 1 to every file it writes: `// dmx: generated from <seed file name> — do not edit.` The marker is the whole ownership protocol: a file that carries it is machine-owned outright, no regions, no author bytes, and byte-exactness ([emission.inline-backend.byte-exactness]) has nothing in it to protect.
- **Never overwrite a human.** A target path that already exists without a dmx marker is somebody's hand-written file, and the driver MUST refuse with `DMX7008` rather than touch it. The same code covers two macro files claiming one name in a single pass, and a macro naming the seed's own file.
- **Stale collection.** After a pass over a seed **in which a macro actually expanded**, any `.dart` file in the seed's directory whose marker names **this seed** and which the pass did not produce MUST be deleted. A dropped table means a dropped file — the generated tree tracks the source of truth in both directions.
- **Nothing ran, nothing is collected.** A pass where no macro expanded MUST NOT write or collect any file, even when the source carries an annotation. An absent worker, an uninstalled `dart`, a crashed process and a checkout without `tool/` all expand nothing, and reading that as "the source of truth dropped everything" would delete a generated tree over a broken toolchain. Deletion requires a macro that ran and did not produce the file.
- **Editable in the ordinary sense.** A macro-authored file is still generated code someone will edit, delete, or revert, and its marker names the seed that produces it. Under `watch` ([execution.modes]), a change to a file carrying the marker MUST re-run the seed the marker names, and a marked file that is deleted MUST be written again. Re-running the seed is the only answer available: the authored file carries no annotation of its own, so a pass over it alone can only ever report "unchanged".
- **Same bar.** Each file's text passes through the one normalizer and MUST parse; an unparseable file fails the build and nothing is written ([dartmacros.pipeline], [validation]). Writes are atomic and no-op-aware ([emission.inline-backend.no-op-writes]), so `watch` does not loop on its own output.
- **Drift.** Under `check` ([execution]), a sibling that would be created, rewritten, or collected is drift on the seed's pass: reported, exit non-zero, nothing written.
- **Inert as input.** A macro-authored file carries no `@dmx`, so later passes leave it untouched; generated output is never re-scanned for triggers ([rendering]).

---

