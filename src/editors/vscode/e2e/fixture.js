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

module.exports = { annotatedClass, document };
