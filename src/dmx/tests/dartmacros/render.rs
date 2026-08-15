//! E2E: a user macro renders with dmx's own Mustache [dartmacros.render].
//!
//! Black-box over the real binary and a real worker. The fixtures here speak
//! the protocol by hand rather than through `package:dmx`, so what is under
//! test is the wire contract itself: the driver must answer a `render` request
//! that arrives while it is waiting for an `expand` reply, and must do it with
//! the engine the built-in catalogue renders with.

use super::{build_and_read, dmx, project};

/// A worker that answers `expand` by asking the driver to render `template`
/// against `context`, then returns whatever came back.
///
/// Deliberately hand-rolled: it writes the request frame, then reads frames
/// until it sees the reply to its own id. That is exactly the interleaving
/// `dmxServeMacros` performs, written out so the test fails if the driver
/// stops servicing the reverse direction.
fn rendering_worker(template: &str, context: &str) -> String {
    format!(
        r#"
import 'dart:convert';
import 'dart:io';

const template = r"""{template}""";

void reply(Map<String, Object?> frame) => stdout.writeln(jsonEncode(frame));

Future<void> main() async {{
  final frames = stdin.transform(utf8.decoder).transform(const LineSplitter());
  final pending = <String, void Function(Map<String, Object?>)>{{}};
  await for (final line in frames) {{
    final Object? message = jsonDecode(line);
    if (message is! Map<String, Object?>) {{
      continue;
    }}
    switch (message['op']) {{
      case 'hello':
        reply({{
          'v': 1,
          'name': 'fixture',
          'version': '0.0.0',
          'contextVersion': 1,
          'ops': ['expand'],
          'macros': ['render'],
        }});
      case 'expand':
        final id = message['id'];
        pending['r1'] = (answer) {{
          final text = answer['text'];
          final error = answer['error'];
          if (text is String) {{
            reply({{
              'v': 1,
              'id': id,
              'text': text,
              'introduced': <String>[],
              'diagnostics': <Object>[],
            }});
          }} else {{
            reply({{
              'v': 1,
              'id': id,
              'refusal': {{'code': 'DMX3999', 'message': '$error'}},
            }});
          }}
        }};
        reply({{
          'v': 1,
          'op': 'render',
          'id': 'r1',
          'name': 'fixture.mustache',
          'template': template,
          'context': jsonDecode(r"""{context}"""),
        }});
      default:
        final id = message['id'];
        final settle = id is String ? pending.remove(id) : null;
        if (settle != null) {{
          settle(message);
        }}
    }}
  }}
}}
"#
    )
}

/// The seed every case here generates from.
const SEED: &str = r"
import 'package:dmx/dmx.dart';

@dmx('render')
class Order {
  const Order({required this.id, required this.total});

  final String id;
  final int total;
}
";

/// [dartmacros.render]: a macro's model reaches Mustache and comes back as
/// members in the region.
#[test]
fn a_macro_renders_its_model_through_dmx() {
    let dir = project(
        &rendering_worker(
            "{{#fields}}\n  String get {{name}}Label => {{{label}}};\n{{/fields}}",
            r#"{"fields":[{"name":"id","label":"'Id'"},{"name":"total","label":"'Total'"}]}"#,
        ),
        "order.dart",
        SEED,
    );
    let generated = build_and_read(&dir, "order.dart");
    assert!(
        generated.contains("String get idLabel => 'Id';"),
        "rendered members missing:\n{generated}"
    );
    assert!(
        generated.contains("String get totalLabel => 'Total';"),
        "rendered members missing:\n{generated}"
    );
}

/// [dartmacros.render]: the triple stache is what keeps a Dart type argument
/// intact, exactly as it does in a built-in's template.
#[test]
fn escaping_matches_the_builtin_dialect() {
    let dir = project(
        &rendering_worker(
            "  // raw {{{type}}} escaped {{type}}",
            r#"{"type":"List<int>"}"#,
        ),
        "order.dart",
        SEED,
    );
    let generated = build_and_read(&dir, "order.dart");
    assert!(
        generated.contains("// raw List<int> escaped List&lt;int&gt;"),
        "escaping differs from the built-in dialect:\n{generated}"
    );
}

/// [dartmacros.render]: an empty list is falsy, so the inverse section runs —
/// Mustache's reading, not JavaScript's.
#[test]
fn mustache_truthiness_governs_a_json_model() {
    let dir = project(
        &rendering_worker(
            "{{#rows}}\n  // row {{name}}\n{{/rows}}\n{{^rows}}\n  // no rows\n{{/rows}}",
            r#"{"rows":[]}"#,
        ),
        "order.dart",
        SEED,
    );
    let generated = build_and_read(&dir, "order.dart");
    assert!(
        generated.contains("// no rows"),
        "an empty list should take the inverse section:\n{generated}"
    );
}

/// [dartmacros.render]: a template that does not compile is answered, not
/// crashed on, and the macro turns that answer into its own diagnostic.
#[test]
fn a_bad_template_comes_back_as_a_refusal() {
    let dir = project(
        &rendering_worker("{{> nowhere}}", r"{}"),
        "order.dart",
        SEED,
    );
    let output = dmx(&dir, &["build", "lib", "--insert-regions"]);
    assert!(
        !output.status.success(),
        "a refused expansion must fail the build"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("DMX7009"),
        "the diagnostic should name the render failure: {stderr}"
    );
    assert!(
        stderr.contains("fixture.mustache"),
        "the diagnostic should name the template: {stderr}"
    );
}
