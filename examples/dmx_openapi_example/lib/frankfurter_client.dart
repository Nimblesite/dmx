// dmx: generated from frankfurter.dart — do not edit.

import 'package:dmx/dmx.dart';

import 'api.dart';

/// Frankfurter is an open-source API for current and historical foreign
/// exchange rates published by central banks.
///
/// Generated from the vendored OpenAPI document for Frankfurter API 2.1.1:
/// one method per `operationId`, with the parameters the document declares and
/// the response schema it promises. Never edit this file.
///
/// The client talks to a [DmxTransport] and nothing else, so a test hands it
/// canned bytes and exercises this very code.
final class FrankfurterClient {
  /// Builds a client over `transport`.
  const FrankfurterClient(this.transport, {this.headers = const {}});

  /// Where the calls go.
  final DmxTransport transport;

  /// Headers added to every request — auth is yours, not the generator's.
  final Map<String, String> headers;

  /// The server the document declares.
  static const String baseUrl = 'https://api.frankfurter.dev/v2';

  /// The version of the API this client was generated from.
  static const String apiVersion = '2.1.1';

  /// Every `operationId` the document declares, in document order.
  static const List<String> operationIds = ['getRates', 'getRate', 'getCurrency', 'getCurrencies', 'getProviders'];

  /// Get exchange rates
  ///
  /// `GET /rates`
  ///
  /// [date] — Specific date (YYYY-MM-DD). Cannot be combined with
  /// from/to.
  ///
  /// [from] — Start of date range (YYYY-MM-DD)
  ///
  /// [to] — End of date range (YYYY-MM-DD). Defaults to today.
  ///
  /// [base] — Base currency (default: EUR)
  ///
  /// [quotes] — Comma-separated list of quote currencies to include
  ///
  /// [providers] — Comma-separated list of data providers to include
  ///
  /// [group] — Downsample rates by time period. Only applies to date
  /// ranges. Allowed: `week`, `month`.
  ///
  /// [expand] — Comma-separated list of optional fields to include per
  /// record. Currently supports `providers`, which adds an array of `{
  /// key, date, rate }` objects per record showing each provider's
  /// individual observation date and rate. Outliers excluded from the
  /// blend (and providers whose rate was overridden by a currency peg)
  /// are flagged with `excluded: true`. The field is omitted on
  /// synthesized peg rows where no provider published the quote. In CSV
  /// output, the `providers` column is encoded as `KEY:RATE` pairs joined
  /// by `|`, with a trailing `*` on excluded entries (e.g.
  /// `ECB:0.92|FED:1.50*`). Allowed: `providers`.
  Future<Result<List<Rate>, ApiError>> getRates({
    String? date,
    String? from,
    String? to,
    String? base,
    String? quotes,
    String? providers,
    String? group,
    String? expand,
  }) async =>
      _send(
        'GET',
        '/rates',
        <String, String>{
          if (date != null) 'date': date,
          if (from != null) 'from': from,
          if (to != null) 'to': to,
          if (base != null) 'base': base,
          if (quotes != null) 'quotes': quotes,
          if (providers != null) 'providers': providers,
          if (group != null) 'group': group,
          if (expand != null) 'expand': expand,
        },
        (body) => dmxListOf(Rate.fromJson)(body, 'List<Rate>'),
      );

  /// Get a single exchange rate pair
  ///
  /// `GET /rate/{base}/{quote}`
  ///
  /// [base] — The `base` path parameter.
  ///
  /// [quote] — The `quote` path parameter.
  ///
  /// [date] — Specific date (YYYY-MM-DD). Cannot be combined with
  /// from/to.
  ///
  /// [providers] — Comma-separated list of data providers to include
  Future<Result<Rate, ApiError>> getRate({
    required String base,
    required String quote,
    String? date,
    String? providers,
  }) async =>
      _send(
        'GET',
        '/rate/${Uri.encodeComponent(base)}/${Uri.encodeComponent(quote)}',
        <String, String>{
          if (date != null) 'date': date,
          if (providers != null) 'providers': providers,
        },
        (body) => Rate.fromJson(body, 'Rate'),
      );

  /// Get a single currency
  ///
  /// `GET /currency/{code}`
  ///
  /// [code] — The `code` path parameter.
  Future<Result<CurrencyDetail, ApiError>> getCurrency({
    required String code,
  }) async =>
      _send(
        'GET',
        '/currency/${Uri.encodeComponent(code)}',
        const <String, String>{},
        (body) => CurrencyDetail.fromJson(body, 'CurrencyDetail'),
      );

  /// Get available currencies
  ///
  /// `GET /currencies`
  ///
  /// [scope] — Set to 'all' to include legacy currencies Allowed: `all`.
  ///
  /// [providers] — Comma-separated list of data providers to include
  Future<Result<List<Currency>, ApiError>> getCurrencies({
    String? scope,
    String? providers,
  }) async =>
      _send(
        'GET',
        '/currencies',
        <String, String>{
          if (scope != null) 'scope': scope,
          if (providers != null) 'providers': providers,
        },
        (body) => dmxListOf(Currency.fromJson)(body, 'List<Currency>'),
      );

  /// Get available data providers
  ///
  /// `GET /providers`
  Future<Result<List<Provider>, ApiError>> getProviders() async =>
      _send(
        'GET',
        '/providers',
        const <String, String>{},
        (body) => dmxListOf(Provider.fromJson)(body, 'List<Provider>'),
      );

  /// The one place a call is actually made: build the URL, send it, classify
  /// the failure, decode the payload.
  ///
  /// Every method above is this, with a different path and a different
  /// decoder — which is exactly the repetition a generator should be absorbing
  /// rather than a person.
  Future<Result<T, ApiError>> _send<T>(
    String method,
    String path,
    Map<String, String> query,
    Result<T, DecodeError> Function(Object? body) decode,
  ) async =>
      switch (await transport.send(DmxRequest(
        method: method,
        url: Uri.parse('$baseUrl$path').replace(
          queryParameters: query.isEmpty ? null : query,
        ),
        headers: <String, String>{
          'accept': 'application/json',
          ...headers,
        },
      ))) {
        Err(error: final failure) => Err(ApiTransportFailure(failure)),
        Ok(value: final response) when !response.isSuccess =>
          Err(ApiStatusFailure(response.status, response.body)),
        Ok(value: final response) => switch (decode(response.body)) {
            Ok(value: final value) => Ok(value),
            Err(error: final failure) => Err(ApiDecodeFailure(failure)),
          },
      };
}
