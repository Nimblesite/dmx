/// Identifier casing for macro authors [context.helpers].
///
/// A macro reading a source of truth outside Dart — a database schema, an
/// OpenAPI document, a wire format — is handed names in that world's casing
/// and has to spell them Dart's way.
///
/// **These are ports of `src/casing.rs`, character for character.** The Rust
/// helpers are what the built-in catalogue names things with, so a macro
/// authored in Dart that cased identifiers its own way would disagree with a
/// built-in about the same field name — in the same file, under the same
/// annotation. `dmxWords` is the single reading of "a word" the rest are
/// derived from, exactly as `casing::words` is on the Rust side.
///
/// Anything the Rust side does not offer is deliberately absent: a macro that
/// wants a further casing is asking for a context variable instead
/// [context.discipline].
library;

/// The words in `name`, in order — the reading every casing here is built on.
///
/// Runs of capitals stay together, so `parseHTTPResponse` is
/// `parse | HTTP | Response` rather than one word per letter. `_`, `-`, ` `,
/// and `$` separate; so does a lower-to-upper transition.
///
/// Mirrors `casing::words`.
List<String> dmxWords(String name) {
  const separators = ['_', '-', ' ', r'$'];
  final characters = name.split('');
  final words = <String>[];
  final word = StringBuffer();
  for (var index = 0; index < characters.length; index++) {
    final character = characters[index];
    if (separators.contains(character)) {
      if (word.isNotEmpty) {
        words.add(word.toString());
        word.clear();
      }
      continue;
    }
    final previous = index == 0 ? '' : characters[index - 1];
    final next = index + 1 < characters.length ? characters[index + 1] : '';
    final startsWord = _isUpper(character) &&
        word.isNotEmpty &&
        (_isLower(previous) || _isDigit(previous) || _isLower(next));
    if (startsWord) {
      words.add(word.toString());
      word.clear();
    }
    word.write(character);
  }
  if (word.isNotEmpty) {
    words.add(word.toString());
  }
  return words;
}

/// The Dart type name for a foreign one: `currency_detail` gives
/// `CurrencyDetail`.
///
/// Each word keeps everything after its first character, so `parseHTTPResponse`
/// gives `ParseHTTPResponse` and an acronym survives. Mirrors `casing::pascal`.
String dmxPascalCase(String name) =>
    [for (final word in dmxWords(name)) _capitalized(word)].join();

/// The Dart identifier for a foreign name: `iso_code` gives `isoCode`.
///
/// This is [dmxPascalCase] with its *first character* lowered — not its first
/// word — which is what `casing::camel` does. `ORDER_ID` therefore gives
/// `oRDERID`; a name already shouting is not quietly reinterpreted.
String dmxCamelCase(String name) => _recasedFirst(
      dmxPascalCase(name),
      (character) => character.toLowerCase(),
    );

/// The Dart file name for a type: `CurrencyDetail` gives `currency_detail`.
///
/// The `.dart` suffix is the caller's business — a macro naming a file adds it
/// itself [dartmacros.files]. Mirrors `casing::snake`.
String dmxSnakeCase(String name) =>
    [for (final word in dmxWords(name)) word.toLowerCase()].join('_');

/// `word` with its first character recased and the rest left alone.
String _recasedFirst(String word, String Function(String) first) =>
    word.isEmpty ? '' : '${first(word.substring(0, 1))}${word.substring(1)}';

/// `code` to `Code`, leaving the rest alone so `HTTP` does not become `Http`.
String _capitalized(String word) =>
    _recasedFirst(word, (character) => character.toUpperCase());

/// Whether `character` is a cased letter in upper case.
bool _isUpper(String character) =>
    character.isNotEmpty &&
    character == character.toUpperCase() &&
    character != character.toLowerCase();

/// Whether `character` is a cased letter in lower case.
bool _isLower(String character) =>
    character.isNotEmpty &&
    character == character.toLowerCase() &&
    character != character.toUpperCase();

/// Whether `character` is an ASCII digit — a word boundary before a capital,
/// so `order2Id` reads as three words the way `casing::words` reads it.
bool _isDigit(String character) =>
    character.length == 1 &&
    character.codeUnitAt(0) >= 0x30 &&
    character.codeUnitAt(0) <= 0x39;
