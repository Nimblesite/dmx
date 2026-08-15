---
layout: layouts/blog.njk
title: "Dart model code generation without part files"
date: 2026-08-14
author: Christian Findlay
tags:
  - posts
  - dart
  - flutter
  - code-generation
category: Code generation
description: dmx generates Dart model code on save inside the annotated class, with no generated part files and a project-controlled output shape.
excerpt: dmx generates copyWith, equality, toString and typed JSON inside the annotated class, with no generated part files.
leadImage:
  src: /assets/images/blog/dmx-code-generation.webp
  alt: Dart code generation adding members inside one source file
  width: 1672
  height: 941
---

dmx is a Dart code generation tool. Its primary use case is model code: `copyWith`, equality, `hashCode`, `toString`, typed JSON, and unions — the work many Flutter projects currently give to Freezed, `json_serializable`, dart_mappable, built_value, and similar packages.

What makes it different is where the generated code goes. Here is a class before saving:

```dart
import 'package:dmx/dmx.dart';

@dmx('model')
class Plain {
  const Plain({required this.id, required this.count, required this.active});

  final String id;
  final int count;
  final bool active;
}
```

Save the file, and the members appear inside that same class, between two markers:

```dart
@dmx('model')
class Plain {
  const Plain({required this.id, required this.count, required this.active});

  final String id;
  final int count;
  final bool active;

  //#region
  static Result<Plain, DecodeError> fromJson(Object? json, [String path = 'Plain']) =>
      switch (json) {
        {
          'id': final String id,
          'count': final int count,
          'active': final bool active,
        } =>
          Ok(Plain(id: id, count: count, active: active)),
        _ => Err(DecodeError(path, 'Plain', json)),
      };

  Map<String, dynamic> toJson() =>
      <String, dynamic>{'id': id, 'count': count, 'active': active};

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Plain &&
          other.id == id &&
          other.count == count &&
          other.active == active);

  @override
  int get hashCode => Object.hash(runtimeType, id, count, active);

  @override
  String toString() => 'Plain(id: $id, count: $count, active: $active)';

  Plain copyWith({String? id, int? count, bool? active}) => Plain(
        id: id ?? this.id,
        count: count ?? this.count,
        active: active ?? this.active,
      );
  //#endregion
}
```

That is the whole file — the six members above are exactly what the shipped `model` template emits, wrapped a little tighter here to fit the page. No `part` directive, no `.g.dart` fragment, no generated mixin the class has to extend, and no delegating factory. You scroll down and read the `fromJson` your app actually runs. Go-to-definition on `copyWith` lands on the line above, not in another file. The diff a reviewer sees puts the field change and the regenerated members side by side.

The decoder returns `Result<Plain, DecodeError>` rather than throwing, because that is what the shipped template emits — and if a project wants something else, it edits the template. More on that below.

## Why generate model code at all

Dart gives you identity equality by default. Two objects with identical fields are not equal unless you write `operator ==` and `hashCode` yourself, as the [`Object.==` documentation](https://api.dart.dev/dart-core/Object/operator_equals.html) spells out. Add copying, a readable `toString`, JSON in both directions, and sealed variants, and a modest app accumulates hundreds of lines nobody wants to write or review.

Flutter's own [JSON guide](https://docs.flutter.dev/data-and-backend/serialization/json) reaches the same conclusion. It says manual serialization becomes tedious and error-prone as a project grows, recommends code generation for medium-to-large projects, and explains why the obvious escape hatch is closed: Dart disables runtime reflection because it defeats tree shaking and inflates application size. The [data-classes request](https://github.com/dart-lang/language/issues/314) has been open for years, full of developers asking the language to absorb this work.

So the demand is real, and Freezed, [`json_serializable`](https://pub.dev/packages/json_serializable), [dart_mappable](https://pub.dev/packages/dart_mappable), and [built_value](https://pub.dev/packages/built_value) answer it. They save an enormous amount of typing, and plenty of teams are happy with them. I am not claiming otherwise, and dmx is not a feature-for-feature replacement for any of them.

But the code those packages generate lands in a separate file, and its shape belongs to the package.

## The complaints are on the record

This is not a niche irritation. Flutter's own triaged issue, [Code generation experience needs improvements](https://github.com/flutter/flutter/issues/63323), collects it: every project has to configure generation and remember to run it, generation delays feedback after an edit, deleting and recreating outputs can produce temporary analyzer errors, each generated class adds `part` declarations and package-specific factory boilerplate, and generated files fill the workspace, search results, and navigation.

That issue was opened in 2020, so read the specifics historically — tooling has moved on since. The categories have not. Developers still describe `.g.dart` proliferation, searches that return generated symbols, and go-to-definition landing in a generated fragment; a [FlutterDev discussion on code generation](https://www.reddit.com/r/FlutterDev/comments/1pn9a8j/what_is_your_opinion_on_code_generation_in/) gathers those complaints along with inflexible generated APIs and indirection nobody asked for.

Then there is the version-control argument, which never ends. Commit the generated files and accept noisy diffs, conflicts, and review churn. Ignore them and every checkout, pull, editor, and CI job has to regenerate before the project is complete. People have been disagreeing about it for years on [Stack Overflow](https://stackoverflow.com/questions/56110386/should-i-commit-generated-code-in-flutter-dart-to-vcs) and in [Flutter community threads](https://www.reddit.com/r/FlutterDev/comments/1861t3j/should_we_add_files_created_with_code_generation/).

Inline generation sidesteps most of that by construction. There is no extra file to navigate, ignore, or fight over, because the generated members live in the file you already track. Macros that intentionally author a complete standalone file — a class per SQLite table, say — are still whole files with explicit ownership, and those you can commit or ignore as you prefer.

## Dart's macros were supposed to fix this, and didn't

The language team saw the problem clearly. In the [Dart 3.4 announcement](https://dart.dev/blog/announcing-dart-3-4) they described existing generators as external tools that complicate the developer experience, and the experimental macro preview was meant to fold generation into the toolchain, visible to analysis and code completion.

It never shipped. In January 2025 the team [stopped work on macros](https://dart.dev/blog/an-update-on-dart-macros-data-serialization). Deep semantic introspection kept slowing static analysis, code completion, and incremental compilation — and incremental compilation is the first stage of Flutter hot reload, so the cost hit the workflow everyone uses all day. They concluded they could not get the experience right in a reasonable timeframe and redirected the effort.

Worth being precise about this: Dart did not ship macros and later deprecate them. It ended an experiment before it became a language feature. The implementation was [removed from the SDK](https://github.com/dart-lang/sdk/issues/60595), and the [current build documentation](https://dart.dev/tools/build_runner) states plainly that Dart's compilers do not support macros. dmx is not that feature resurrected — it is an external tool that runs when you save. Narrower work continues on [augmentations](https://github.com/dart-lang/language/issues/4154), serialization, analyzer performance, and generator speed, but today, generated Dart comes from an external tool.

## What about build_runner?

`build_runner` is Dart's supported general-purpose build system, it is actively maintained, and it has a watch mode. Its [changelog](https://github.com/dart-lang/build/blob/master/build_runner/CHANGELOG.md) records real improvements in 2.13 and 2.14 — faster incremental builds, AOT-compiled builders, fewer unchanged writes, better startup — and 2.7 removed the old conflicting-output prompt. Generator scalability is tracked as [high-priority performance work](https://github.com/dart-lang/build/issues/3800).

I want to be careful here, because it is easy to blame the build system for decisions it did not make. The `part` files and the fixed generated APIs come from the model generators layered on top, not from `build_runner` itself. If you are happy with your generator's output shape and happy with `.g.dart` files, `build_runner` is not the thing standing in your way, and dmx has nothing to sell you.

dmx is for the case where the output shape and the extra files *are* the problem.

## How it works

When you save, the VS Code extension's watcher picks up the change, regenerates only the affected declarations, and skips writes that would not change anything. In another editor you run `dmx watch` once and leave it running. Either way there is no build command in your loop.

Inside, the pipeline is deliberately boring. tree-sitter parses the Dart into a concrete syntax tree. A Rust context builder works out the completed expressions — every decode, encode, equality, hash, and copy expression, with nullability, collections, and nested generics already resolved. A Mustache template arranges those finished strings into Dart. The candidate file is reparsed and validated. Only then does the emitter splice the result between the markers.

That split is the reason templates are usable. The template does not need to know that `List<Map<String, Product?>>` decodes differently from `int`; the context builder already answered that. So the template stays legible:

{% raw %}
```mustache
  {{className}} copyWith({
{{#fields}}
    {{{copyParam}}},
{{/fields}}
  }) =>
      {{className}}(
```
{% endraw %}

Change that file and every model in the project adopts your conventions — your constructor style, your naming, your JSON failure behaviour. The built-ins that ship with dmx (`model`, `union`, `enum`, `diff`, `lerp`, `validate`, `table`, `route`, `cli`, `fake`, `restClient`) are the same templates, just compiled into the binary.

When a template cannot answer the question, you write a macro in Dart. A macro sees typed parsed declarations and their siblings, and it can read whatever source of truth the project has — a SQLite schema, an OpenAPI document, a config file — then generate members or entire files. It can return Dart directly, which for a small macro is usually the right call, or hand its model to a Mustache template and let dmx render it with the same engine the built-ins use. That is how the generation logic and the output shape end up in separate files: the macro answers what only your project knows, the template decides what the Dart looks like.

Two honest limits. Macros currently receive typed parsed declarations, not the analyzer's full semantic model, so there is no full type inference to lean on yet. And validation is what stands between a template bug and a broken repository: dmx reparses the entire candidate file before writing, and on failure it emits a diagnostic and leaves your source exactly as it was. Bytes outside the region are never touched.

| Typical generated-model package | dmx |
| --- | --- |
| Generated members in `.freezed.dart`, `.g.dart`, or another `part` fragment | Generated members inside the annotated declaration |
| The package's generated API and structure | A Mustache template the project edits |
| Write a generator package for custom behaviour | Write a macro in Dart, and render its Mustache template |
| Fragments validated by the surrounding toolchain | The complete candidate file parsed and validated before writing |

## Try it before installing anything

The [playground](/playground.html) runs the production generator in your browser. Edit the Dart, edit the Mustache template, watch the output change. Nothing installs, and neither input leaves the tab — it is the actual generator compiled to WebAssembly, not a canned demo.

If it fits how you want to work, the [installation guide](/docs/) takes it from there, and the source is at [Nimblesite/dmx](https://github.com/Nimblesite/dmx).
