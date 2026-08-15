# dmx — Built-in Model Macro

Part of the [dmx specification](SPEC.md).

## [model] Built-in Macro: `@dmx('model')`

### [model.copywith] `copyWith` (normative)

- Non-nullable `T x` → parameter `T? x`, applied `x ?? this.x`
- Nullable `T? x` → parameter `Object? x = _$unset`, applied `identical(x, _$unset) ? this.x : x as T?`

`copyWith(email: null)` clears; `copyWith()` preserves. A `copyWith` that cannot express "set to null" is a defect, not a simplification. Under `inline`, `_$unset` is a `static const` member of the target class, requiring no top-level declaration.

### [model.equality] Equality (normative)

`DeepCollectionEquality` for collection-typed fields, `==` otherwise. `hashCode` MUST include `runtimeType` first. Beyond 20 components, MUST use `Object.hashAll([…])`.

### [model.json-codec] JSON codec table (normative)

Decoding is **total**. `fromJson` MUST return `Result<T, DecodeError>` and MUST NOT throw, cast (`as`), or force (`!`):

- `Ok<T, DecodeError>` carries the model.
- `Err<T, DecodeError>` carries a `DecodeError(path, expected, actual)` naming where the failure happened — `User.tags[1]`, `User.scores[math]`.

The failure is a type parameter, not a fixed shape, so the same `Result<T, E>` composes with consumer code that chose its own `E`. An enclosing decode re-wraps an inner `Err` **unchanged**: the path names the origin of the failure, never the point at which it was observed.

A required field whose JSON representation *is* its Dart representation is bound directly by the map pattern and needs no `Result`. Every other field contributes one `Result` to a record, and one exhaustive record pattern turns the tuple into `Ok(Model(…))` or re-wraps the first `Err`.

Produces `resultExpr` / `encodeExpr` in Rust; templates never see this table.

| Type | Decode | Encode |
|---|---|---|
| `String`, `int`, `bool`, `num`, `dynamic`, `Object` | bound by the map pattern; `Ok(v)` under a decoder | `instance.f` |
| `double` | `Ok(v.toDouble())` — JSON `1` arrives as an `int`, so the shape is `num` | `instance.f` |
| `DateTime` | `switch (DateTime.tryParse(v)) { final DateTime parsed => Ok<DateTime, DecodeError>(parsed), null => Err<DateTime, DecodeError>(DecodeError(path, 'DateTime', v)) }` | `.toIso8601String()` |
| `Uri`, `BigInt` | as `DateTime`, via `tryParse` | `.toString()` |
| `Duration` | `Ok(Duration(microseconds: v))` | `.inMicroseconds` |
| nullable `T?` | `dmxNullable<T>(json[k], path, ⟨decode T⟩)` — absent or null is `Ok(null)` | `?.⟨encode T⟩` |
| `List<E>` | `dmxList<E>(v, path, ⟨decode E⟩)` — `Err` at the first bad element | `.map((e) => ⟨encode E⟩).toList()` |
| `Set<E>` | `dmxSet<E>(v, path, ⟨decode E⟩)` | `.toList()` |
| `Map<String,V>` | `dmxMap<V>(v, path, ⟨decode V⟩)` — `Err` at the first bad value | symmetric |
| `Map<K,V>`, K ≠ String | `DMX2101` unless a converter is given | — |
| nested model | `M.fromJson(v)`, itself a `Result` | `.toJson()` |
| shape mismatch | `Err(DecodeError(path, ⟨expected⟩, v))` | — |
| enum | `Result`-returning lookup over the value set | the value set |
| type parameter | `fromJsonT(json[k])`, returning a `Result` | `toJsonT(instance.f)` |
| unresolved | `DMX2102` | — |
| `@dmx('key', {'converter': C})` | `const C().fromJson(json[k])`, returning a `Result` | `const C().toJson(instance.f)` |

Nesting recurses without a fixed depth limit.

### [model.deferred] Deferred to v1.1+

Sealed/union hierarchies with discriminators; `@dmx('key', {'defaultValue': …})` on decode; `unknownKeys: error`; `includeIfNull: false`; primary-constructor targets.

---

