import 'package:dmx/dmx.dart';

@dmx('model')
class Renamed {
  const Renamed({required this.createdAt, required this.userName, this.secret});

  @dmx('key', {'name': 'created_at'})
  final DateTime createdAt;
  @dmx('key', {'name': 'user_name'})
  final String userName;
  @dmx('key', {'ignore': true})
  final String? secret;
}
