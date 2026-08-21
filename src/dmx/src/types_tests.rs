//! The codec table, held to the Dart it must produce [model.json-codec].
//!
//! A separate file only because types.rs is at the 500-line ceiling.

use super::*;

fn parse(ty: &str) -> DartType {
    DartType::parse(ty).unwrap()
}

/// [model.json-codec]: decoding is total — no `throw`, no `as`, no `!`.
#[test]
fn decoding_never_throws_or_casts() {
    for ty in [
        "int",
        "String",
        "DateTime",
        "List<int>",
        "Set<String>",
        "Address",
    ] {
        let out = decode_bound(&parse(ty), "value", "'$path.f'", 0, Runtime::IN_CLASS).unwrap();
        for forbidden in ["throw", " as ", "!"] {
            assert!(
                !out.contains(forbidden),
                "`{forbidden}` in decode of {ty}: {out}"
            );
        }
    }
    for ty in ["String?", "List<String>?", "Map<String, int>?"] {
        let out = decoder(&parse(ty), 0, Runtime::IN_CLASS).unwrap();
        for forbidden in ["throw", " as ", "!"] {
            assert!(
                !out.contains(forbidden),
                "`{forbidden}` in decoder for {ty}"
            );
        }
    }
}

#[test]
fn decode_shapes() {
    assert_eq!(
        decode_bound(&parse("int"), "age", "'$path.age'", 0, Runtime::IN_CLASS).unwrap(),
        "Ok(age)"
    );
    assert_eq!(
        decode_bound(&parse("DateTime"), "at", "'$path.at'", 0, Runtime::IN_CLASS).unwrap(),
        "switch (DateTime.tryParse(at)) { \
         final DateTime parsed => Ok<DateTime, DecodeError>(parsed), \
         null => Err<DateTime, DecodeError>(DecodeError('$path.at', 'DateTime', at)) }"
    );
    assert_eq!(
        decode_bound(&parse("Address"), "a", "'$path.a'", 0, Runtime::IN_CLASS).unwrap(),
        "Address.fromJson(a, '$path.a')"
    );
    assert_eq!(
        decode_bound(
            &parse("List<String>"),
            "tags",
            "'$path.tags'",
            0,
            Runtime::IN_CLASS
        )
        .unwrap(),
        "dmxList<String>(tags, '$path.tags', (value, path) => switch (value) {\n\
         \x20 final String value => Ok(value),\n\
         \x20 _ => Err(DecodeError(path, 'String', value)),\n\
         })"
    );
    // Nested nullability composes through dmxNullable.
    assert!(
        decoder(&parse("List<String?>"), 0, Runtime::IN_CLASS)
            .unwrap()
            .contains("dmxNullable<String>(value, path,")
    );
}

/// A declared type is its own decoder, whatever kind of declaration it is.
#[test]
fn declared_types_decode_themselves() {
    assert_eq!(
        decoder(&parse("Address"), 0, Runtime::IN_CLASS).unwrap(),
        "Address.fromJson"
    );
    // [typediagram.canonical]: a class that keeps its codec on an extension
    // is reached through the extension, everywhere the table names it.
    assert_eq!(
        decoder(&parse("Address"), 0, Runtime::PREFIXED).unwrap(),
        "AddressJson.fromJson"
    );
    assert_eq!(
        decode_bound(&parse("Address"), "a", "'$path.a'", 0, Runtime::PREFIXED).unwrap(),
        "AddressJson.fromJson(a, '$path.a')"
    );
    assert_eq!(
        decode_bound(
            &parse("List<Address>"),
            "xs",
            "'$path.xs'",
            0,
            Runtime::PREFIXED
        )
        .unwrap(),
        "dmx.dmxList<Address>(xs, '$path.xs', AddressJson.fromJson)"
    );
    assert_eq!(json_shape(&parse("Address")), "Object?");
    assert_eq!(
        decode_bound(
            &parse("List<Status>"),
            "s",
            "'$path.s'",
            0,
            Runtime::IN_CLASS
        )
        .unwrap(),
        "dmxList<Status>(s, '$path.s', Status.fromJson)"
    );
}

/// The JSON shape a map pattern must bind before decoding [model.json-codec].
#[test]
fn json_shapes() {
    assert_eq!(json_shape(&parse("DateTime")), "String");
    assert_eq!(json_shape(&parse("List<Address>")), "List<dynamic>");
    assert_eq!(
        json_shape(&parse("Map<String, int>")),
        "Map<String, dynamic>"
    );
    assert_eq!(json_shape(&parse("double")), "num");
    assert_eq!(json_shape(&parse("String")), "String");
}

#[test]
fn encode_expressions() {
    let encode_of = |ty: &str, e: &str| encode(&parse(ty), e, 0);
    assert_eq!(encode_of("List<String>", "tags"), "tags");
    assert_eq!(encode_of("Set<int>", "ids"), "ids.toList()");
    assert_eq!(encode_of("Address?", "home"), "home?.toJson()");
    assert_eq!(encode_of("DateTime", "at"), "at.toIso8601String()");
    assert_eq!(
        encode_of("List<Address>", "stops"),
        "stops.map((e0) => e0.toJson()).toList()"
    );
}
