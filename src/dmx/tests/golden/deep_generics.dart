import 'package:dmx/dmx.dart';

@dmx('model')
class Leaf {
  const Leaf({required this.name});

  final String name;
}

@dmx('model')
class Tree {
  const Tree({
    required this.grid,
    required this.groups,
    required this.leaves,
  });

  final List<List<int>> grid;
  final Map<String, List<Leaf>> groups;
  final Set<Leaf> leaves;
}
