import 'package:dmx/dmx.dart';

@dmx('model')
class Collide {
  const Collide({required this.other, required this.value});

  final String other;
  final int value;
}
