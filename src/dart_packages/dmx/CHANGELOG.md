# Changelog

<!-- Entries land under `## Unreleased` as they are written. The tag decides the
     version, and the release renames this heading to it [release.version]. -->

## Unreleased

- No API changes. The README linked the extension's Marketplace page with the
  publisher in lowercase, which 404s; it is `Nimblesite.dmx`.

## 0.2.0

- No API changes. `lib/` is identical to 0.1.0 apart from the version constant.
- The tag names the version, and the release stamps it [release.version].
- README rewritten in plainer language.

## 0.1.0

First release.

- `@dmx('macro', {…})` — the one annotation that triggers generation
  [surface.annotations]. It carries a macro name and plain-data arguments, and
  runs nothing at runtime.
- The runtime generated code composes with: `Result`/`Ok`/`Err`, `DecodeError`
  with a dotted path to the failure, the `dmxString`/`dmxInt`/`dmxList`/`dmxMap`
  decoders [model.json-codec], `DmxPatch`/`DmxKeep`/`DmxTo` for `copyWith`
  [model.copywith], and `dmxDeepEquals`/`dmxDeepHash` [model.equality].
- `package:dmx/macros.dart` — write a macro in Dart [dartmacros.api]. Extend
  `DmxMacro`, read `DmxInvocation.declaration`, return a `DmxFragment`, a
  `DmxGeneratedFile`, or a `DmxRefusal`, and serve it with `dmxServeMacros`.
- A macro can hand its model to a Mustache template through
  `invocation.templates.render(…)` and get back rendered Dart from the same
  engine the built-in macros use [dartmacros.render].
- `DmxTransport`, `DmxRequest` and `DmxResponse`, which generated REST clients
  call so the app chooses its own HTTP client.
- `DmxPackage.version` reports what this package's pubspec declares, generated
  by the package's own macro worker rather than typed twice [release.version].
