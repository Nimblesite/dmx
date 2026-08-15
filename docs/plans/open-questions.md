# dmx — Open Questions

Part of the [dmx implementation plan](PLAN.md).

## [questions] Open Questions

| # | Question | Blocking |
|---|---|---|
| Q1 | Does tree-sitter-dart report comment token positions reliably enough for [emission.inline-backend.region-location] across the C1 corpus? The whole `inline` backend rests on this. | **Yes** |
| Q2 | Should `build` insert regions with an opt-out rather than requiring `fix`? Ergonomics vs G8. | **Yes** |
| Q3 | Index the current package only, or the whole `package_config.json` graph? | **Yes** |
| Q4 | Is Mustache sufficient for all built-in templates? Settle by writing them in Mustache first. | **Yes** |
| Q5 | Does an in-class `static const Object? _$unset = Object();` sentinel behave identically to the top-level version across all const-constructor cases? | **Yes** |
| Q6 | Does the analyzer pick up in-place file edits promptly under `watch`, or does region rewriting cause visible churn? | Empirical |
| Q7 | How badly does `//#region` folding degrade in editors other than VS Code and IntelliJ? | No |
| Q8 | Should `git-filter` ship as a documented recipe or as a `dmx setup-git` command that writes the config? | No |
| Q9 | Is an embedded Dart-subset interpreter ([extensions.dart-language]) worth building, or is the AOT worker permanently sufficient? | No |
