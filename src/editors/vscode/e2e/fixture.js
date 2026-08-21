'use strict';

// The one shape every fixture class takes [editor.extension.e2e]: annotated,
// one string field, and NO region — inserting the divider is the extension's
// job, and a fixture that arrives with one would hide it not happening.

function annotatedClass(name, field) {
  return `import 'package:dmx/dmx.dart';

@dmx('model')
class ${name} {
  const ${name}({required this.${field}});

  final String ${field};
}
`;
}

// A `*.dmx.md` document [typediagram.documents]: one definition, one bound
// template, and prose around both. There is no Dart source behind it — the
// point of the fixture is that the extension generates from a Markdown file
// with nothing annotated anywhere.

function document(typeName, fieldName) {
  return `# ${typeName}

The definition below is the source of truth.

\`\`\`typeDiagram
type ${typeName} {
  ${fieldName}: String
}
\`\`\`

\`\`\`mustache {"dmx":{"output":"lib/${typeName.toLowerCase()}.dart"}}
{{#declarations}}
final class {{name}} {
  const {{name}}({{{constructorParameters}}});
{{#fields}}

  final {{{dartType}}} {{name}};
{{/fields}}
}
{{/declarations}}
\`\`\`

That is the whole document.
`;
}

// A standalone definition and the template beside it [typediagram.standalone]:
// files, nothing embedded in anything, and no Dart source of truth. The
// template carries no metadata at all — the convention answers both questions
// it could ask — and it takes the canonical model template's place, which is
// what makes this fixture about the extension's wiring rather than about what
// dmx generates [typediagram.canonical].

function definition(typeName, fieldName) {
  return `# ${typeName}, and nothing else in this file.
type ${typeName} {
  ${fieldName}: String
}
`;
}

function template() {
  return `{{#declarations}}
final class {{name}} {
  const {{name}}({{{constructorParameters}}});
{{#fields}}

  final {{{dartType}}} {{name}};
{{/fields}}
}
{{/declarations}}
`;
}

module.exports = { annotatedClass, definition, document, template };
