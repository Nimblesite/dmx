/// The seam a generated REST client is written against [catalogue.rest].
///
/// There is no `package:http` here, and there is no `dart:io` either. A
/// generated client talks to [DmxTransport] and nothing else, which is what
/// makes it testable without a mock framework: a test supplies a transport
/// that returns canned bytes, and the code under test is the real client.
library;

import '../dmx.dart';

/// One outbound call, fully built by generated code.
class DmxRequest {
  final String method;
  final Uri url;
  final Map<String, String> headers;
  final Object? body;

  const DmxRequest({
    required this.method,
    required this.url,
    this.headers = const <String, String>{},
    this.body,
  });

  @override
  String toString() => '$method $url';
}

/// What came back. The body stays `Object?` — decoding it is the generated
/// method's job, and it does that with the same `fromJson` everything else
/// uses.
class DmxResponse {
  final int status;
  final Object? body;

  const DmxResponse({required this.status, this.body});

  bool get isSuccess => status >= 200 && status < 300;
}

/// The one thing you implement. Async, fallible, and returning a `Result` —
/// so a generated client has nothing to catch.
abstract class DmxTransport {
  Future<Result<DmxResponse, TransportError>> send(DmxRequest request);
}

/// A call that did not produce a usable response.
class TransportError {
  final String message;
  final int? status;

  const TransportError(this.message, {this.status});

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is TransportError &&
          other.message == message &&
          other.status == status);

  @override
  int get hashCode => Object.hash(runtimeType, message, status);

  @override
  String toString() => switch (status) {
        final int status => 'TransportError($status: $message)',
        null => 'TransportError($message)',
      };
}

/// Everything that can go wrong on the way to a decoded value: the call, or
/// the payload. Sealed, so handling both is checked rather than remembered.
sealed class ApiError {
  const ApiError();
}

final class ApiTransportFailure extends ApiError {
  final TransportError error;

  const ApiTransportFailure(this.error);

  @override
  String toString() => 'ApiTransportFailure($error)';
}

final class ApiDecodeFailure extends ApiError {
  final DecodeError error;

  const ApiDecodeFailure(this.error);

  @override
  String toString() => 'ApiDecodeFailure($error)';
}

/// A non-2xx response, with whatever the server said about it.
final class ApiStatusFailure extends ApiError {
  final int status;
  final Object? body;

  const ApiStatusFailure(this.status, this.body);

  @override
  String toString() => 'ApiStatusFailure($status)';
}
