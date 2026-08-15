import 'package:dmx/dmx.dart';

@dmx('model')
class Config {
  const Config({this.host, this.port, this.debug});

  final String? host;
  final int? port;
  final bool? debug;
}
