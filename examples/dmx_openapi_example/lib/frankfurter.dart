// One of TWO hand-written files in lib/. Every other file beside it — the
// models, the client, and `api.dart` — is authored, NAMED, and kept current by
// `tool/dmx/macros.dart` from the OpenAPI document at
// `tool/dmx/frankfurter.openapi.json`.
//
// The document is the source of truth. Never edit a generated file: change the
// document, or change the Mustache template that lays the output out, and
// rebuild.

import 'package:dmx/dmx.dart';

/// A whole API, as one annotation.
///
/// The macro reads every schema and every `operationId` out of the vendored
/// OpenAPI document, writes one class per schema in a file it names itself,
/// writes the client, writes the barrel, and leaves the manifest — what the
/// document declares, and where each class went — between the dividers below.
///
/// Nothing about the API is typed here. Not an endpoint, not a parameter, not
/// a response type, not a file name.
@dmx('openApiClient')
class Frankfurter {
  //#region
  /// The API this was generated from.
  static const String title = 'Frankfurter API';

  /// The version the document declares.
  static const String apiVersion = '2.1.1';

  /// The server the document declares, and the client's base URL.
  static const String baseUrl = 'https://api.frankfurter.dev/v2';

  /// Every `operationId` the document declares, in document order.
  static const List<String> operationIds = ['getRates', 'getRate', 'getCurrency', 'getCurrencies', 'getProviders'];

  /// Every schema a class was generated for, the macro's own inline
  /// classes included.
  static const List<String> schemaNames = ['Rate', 'Currency', 'CurrencyDetail', 'Provider', 'RateProvider', 'CurrencyDetailPeg'];

  /// The generated file each class lives in, keyed by schema.
  static const Map<String, String> schemaFiles = {
    'Rate': 'rate.dart',
    'Currency': 'currency.dart',
    'CurrencyDetail': 'currency_detail.dart',
    'Provider': 'provider.dart',
    'RateProvider': 'rate_provider.dart',
    'CurrencyDetailPeg': 'currency_detail_peg.dart',
  };
  //#endregion
}
