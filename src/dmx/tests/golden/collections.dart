import 'package:dmx/dmx.dart';

@dmx('model')
class Bag {
  const Bag({
    required this.tags,
    required this.ids,
    required this.scores,
  });

  final List<String> tags;
  final Set<int> ids;
  final Map<String, double> scores;
}
