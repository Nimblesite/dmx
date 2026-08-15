import 'package:dmx/dmx.dart';

@dmx('model')
class Plain {
  const Plain({required this.id, required this.count, required this.active});

  final String id;
  final int count;
  final bool active;
}
