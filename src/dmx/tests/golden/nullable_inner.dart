import 'package:dmx/dmx.dart';

@dmx('model')
class Node {
  const Node({required this.label});

  final String label;
}

@dmx('model')
class Holder {
  const Holder({
    required this.maybeNames,
    this.child,
    this.children,
  });

  final List<String?> maybeNames;
  final Node? child;
  final List<Node>? children;
}
