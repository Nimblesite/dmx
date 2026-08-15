// The second and last hand-written file in lib/.
//
// A generated client talks to a `DmxTransport` and nothing else, so this is
// where `dart:io` lives — not in generated code, and not behind a mock
// framework. A test that wants canned bytes implements the same interface.

import 'dart:convert';
import 'dart:io';

import 'package:dmx/dmx.dart';

/// A [DmxTransport] over `dart:io`, so the example needs no HTTP package.
///
/// Everything that can go wrong on a network — DNS, TLS, a timeout, a body
/// that is not JSON — comes back as a [TransportError]. Nothing throws past
/// this class, which is what lets the generated client be a pure `switch` over
/// a `Result`.
final class HttpTransport implements DmxTransport {
  /// Builds a transport. `timeout` bounds a single call.
  HttpTransport({Duration timeout = const Duration(seconds: 30)})
      : _client = HttpClient()..connectionTimeout = timeout,
        _timeout = timeout;

  final HttpClient _client;
  final Duration _timeout;

  @override
  Future<Result<DmxResponse, TransportError>> send(DmxRequest request) async {
    try {
      final outbound = await _client
          .openUrl(request.method, request.url)
          .timeout(_timeout);
      for (final MapEntry(:key, :value) in request.headers.entries) {
        outbound.headers.set(key, value);
      }
      final response = await outbound.close().timeout(_timeout);
      final body = await response
          .transform(utf8.decoder)
          .join()
          .timeout(_timeout);
      return Ok(
        DmxResponse(status: response.statusCode, body: _decode(body)),
      );
    } on Object catch (failure) {
      // The one place an exception is allowed to exist in this example: at the
      // edge, being turned into a value. `dart:io` throws; everything above
      // this line does not.
      return Err(TransportError('$failure'));
    }
  }

  /// The body as JSON, or as the raw text when it is not JSON at all — an
  /// error page from a proxy, say, which the client reports as a status
  /// failure with the text intact rather than as a decode failure.
  Object? _decode(String body) {
    if (body.isEmpty) {
      return null;
    }
    try {
      return jsonDecode(body);
    } on FormatException {
      return body;
    }
  }

  /// Releases the underlying client's sockets.
  void close() => _client.close();
}
