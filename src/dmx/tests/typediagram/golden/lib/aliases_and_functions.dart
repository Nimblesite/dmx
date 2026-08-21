// dmx: generated from models/aliases-and-functions.td — do not edit.
// dmx: rendered through the canonical model template, definition fc1a006cd8bfa5cd, template 5fba7c04728545cb, context v1, dmx 0.0.0.

// Generated from models/aliases-and-functions.td. Edit the definition, not this file.

import 'package:dmx/dmx.dart' as dmx;

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

/// Request — an immutable value from the diagram.
final class Request {
  /// Every field, in the order the diagram declares them.
  const Request({required this.url});

  /// The `url` field, declared as `String`.
  final String url;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Request &&
          other.url == url);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        url,
      );

  @override
  String toString() => 'Request(url: $url)';

  /// A copy of this value with the named fields replaced.
  Request copyWith({
    String? url,
  }) =>
      Request(
        url: url ?? this.url,
      );
}

/// JSON for [Request].
extension RequestJson on Request {
  /// Decodes a `Request` from a JSON value, or says why it could not.
  static dmx.Result<Request, dmx.DecodeError> fromJson(Object? json, [String path = 'Request']) =>
      switch (json) {
        {
          'url': final String url,
        } =>
          dmx.Ok(Request(
            url: url,
          )),
        _ => dmx.Err(dmx.DecodeError(path, 'Request', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
        'url': url,
      };
}

/// Response — an immutable value from the diagram.
final class Response {
  /// Every field, in the order the diagram declares them.
  const Response({required this.status});

  /// The `status` field, declared as `Int`.
  final int status;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Response &&
          other.status == status);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        status,
      );

  @override
  String toString() => 'Response(status: $status)';

  /// A copy of this value with the named fields replaced.
  Response copyWith({
    int? status,
  }) =>
      Response(
        status: status ?? this.status,
      );
}

/// JSON for [Response].
extension ResponseJson on Response {
  /// Decodes a `Response` from a JSON value, or says why it could not.
  static dmx.Result<Response, dmx.DecodeError> fromJson(Object? json, [String path = 'Response']) =>
      switch (json) {
        {
          'status': final int status,
        } =>
          dmx.Ok(Response(
            status: status,
          )),
        _ => dmx.Err(dmx.DecodeError(path, 'Response', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
        'status': status,
      };
}
