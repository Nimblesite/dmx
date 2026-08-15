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

module.exports = { annotatedClass };
