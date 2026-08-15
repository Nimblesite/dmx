import 'package:dmx/dmx.dart';

@dmx('model', {'copyWith': false, 'toString': false})
class JsonOnly {
  const JsonOnly({required this.id});

  final String id;
}

@dmx('model', {'json': false, 'equality': false})
class NoJson {
  const NoJson({required this.id});

  final String id;
}
