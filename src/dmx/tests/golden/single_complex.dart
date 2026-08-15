import 'package:dmx/dmx.dart';

@dmx('model')
class Stamped {
  const Stamped({required this.id, required this.at});

  final String id;
  final DateTime at;
}
