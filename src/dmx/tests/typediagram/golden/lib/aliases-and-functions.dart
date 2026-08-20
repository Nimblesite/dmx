// dmx: generated from docs/aliases-and-functions.dmx.md — do not edit.
// dmx: group 1, fences 1/2, definition fc1a006cd8bfa5cd, template ebcc1789a3d0fa99, context v1, dmx 0.0.0.

// Generated from docs/aliases-and-functions.dmx.md. Edit the diagram, not this file.

/// `Email` as the diagram declares it.
typedef Email = String;

/// `UserId` as the diagram declares it.
typedef UserId = String;

/// `Callback` as the diagram declares it.
typedef Callback = String?;

/// `Index` as the diagram declares it.
typedef Index<K> = Map<K, List<Email>>;

/// Signature 0 of `fetch`, as the diagram declares it.
typedef Fetch<T> = Response Function(Request request, T? fallback);

/// Signature 0 of `store`, as the diagram declares it.
typedef Store = Future<void> Function(Request item);

/// Signature 0 of `read`, as the diagram declares it.
typedef Read0 = List<int> Function(String path);

/// Signature 1 of `read`, as the diagram declares it.
typedef Read1 = Future<List<int>> Function(String path, double timeout);

/// Signature 0 of `drain`, as the diagram declares it.
typedef Drain0 = void Function();

/// Signature 1 of `drain`, as the diagram declares it.
typedef Drain1 = Future<int> Function(int limit);

/// Signature 0 of `nothing`, as the diagram declares it.
typedef Nothing = void Function();

/// Request — a record from the diagram.
final class Request {
  /// Every field, in the order the diagram declares them.
  const Request({required this.url});

  /// The `url` field, declared as `String`.
  final String url;
}

/// Response — a record from the diagram.
final class Response {
  /// Every field, in the order the diagram declares them.
  const Response({required this.status});

  /// The `status` field, declared as `Int`.
  final int status;
}
