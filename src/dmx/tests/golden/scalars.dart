import 'package:dmx/dmx.dart';

@dmx('model')
class Scalars {
  const Scalars({
    required this.when,
    required this.where,
    required this.huge,
    required this.howLong,
    required this.ratio,
    required this.count,
  });

  final DateTime when;
  final Uri where;
  final BigInt huge;
  final Duration howLong;
  final double ratio;
  final num count;
}
