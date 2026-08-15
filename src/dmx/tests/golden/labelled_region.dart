import 'package:dmx/dmx.dart';

@dmx('model')
class Coexist {
  const Coexist({required this.id});

  final String id;

  //#region Helpers
  String get shout => id.toUpperCase();
  //#endregion

  //#region
  //#endregion
}
