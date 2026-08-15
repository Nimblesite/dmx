// GENERATED REGIONS ARE MACHINE-OWNED. Edit above the divider, run `dmx build`.
//
// `@dmx('restClient')` [catalogue.rest] — the HTTP layer nobody should hand-write.
//
// The interface is hand-written and readable. The implementation — build the
// URL, set the headers, encode the body, check the status, decode the payload,
// classify the failure — is the same eleven lines per endpoint, written once
// per endpoint, forever, in every app. That is exactly the work a generator
// should be doing.
//
// Note what is *not* here: no `package:http`, no `dart:io`, no mock framework.
// The client talks to a `DmxTransport`, so a test hands it a transport that
// returns canned payloads and exercises the real generated code.

import 'package:dmx/dmx.dart';

import 'catalog.dart';
import 'orders.dart';
import 'payments.dart';

/// What the storefront backend offers. Hand-written, above the divider, and
/// the only thing a caller ever needs to read.
abstract interface class StorefrontApi {
  @dmx('get', {'path': '/products/{id}'})
  Future<Result<Product, ApiError>> product(String id);

  @dmx('get', {'path': '/products'})
  Future<Result<List<Product>, ApiError>> search({
    @dmx('query') required String q,
    @dmx('query') int page = 1,
  });

  @dmx('post', {'path': '/orders'})
  Future<Result<Placed, ApiError>> placeOrder(@dmx('body') Draft draft);

  @dmx('post', {'path': '/orders/{orderId}/refund'})
  Future<Result<Cancelled, ApiError>> refund(
    String orderId,
    @dmx('query') RefundReason reason,
  );

  @dmx('delete', {'path': '/carts/{cartId}'})
  Future<Result<void, ApiError>> abandonCart(String cartId);
}

/// The generated client. `@dmx('restClient')` reads the sibling interface this class
/// `implements` and writes one method per binding [frontend.name-index].
@dmx('restClient', {'baseUrl': 'https://api.storefront.example/v1'})
class StorefrontClient implements StorefrontApi {
  const StorefrontClient(this.transport, {this.headers = const {}});

  final DmxTransport transport;

  /// Hand-written and untouched: auth headers are yours, not the generator's.
  final Map<String, String> headers;

  //#region
  static const String baseUrl = 'https://api.storefront.example/v1';

  static const Map<String, String> defaultHeaders = <String, String>{
    'accept': 'application/json',
  };

  @override
  Future<Result<Product, ApiError>> product(String id) async =>
      switch (await transport.send(DmxRequest(
        method: 'GET',
        url: Uri.parse('$baseUrl/products/$id'),
        headers: <String, String>{
          ...defaultHeaders,
          ...headers,
        },
      ))) {
        Err(error: final e) => Err(ApiTransportFailure(e)),
        Ok(value: final response) when !response.isSuccess =>
          Err(ApiStatusFailure(response.status, response.body)),
        Ok(value: final response) => switch (Product.fromJson(response.body, 'product')) {
            Ok(value: final value) => Ok(value),
            Err(error: final e) => Err(ApiDecodeFailure(e)),
          },
      };

  @override
  Future<Result<List<Product>, ApiError>> search({required String q, int page = 1}) async =>
      switch (await transport.send(DmxRequest(
        method: 'GET',
        url: Uri.parse('$baseUrl/products').replace(queryParameters: dmxQuery(<String, String?>{'q': q, 'page': '$page'})),
        headers: <String, String>{
          ...defaultHeaders,
          ...headers,
        },
      ))) {
        Err(error: final e) => Err(ApiTransportFailure(e)),
        Ok(value: final response) when !response.isSuccess =>
          Err(ApiStatusFailure(response.status, response.body)),
        Ok(value: final response) => switch (switch (response.body) { final List<dynamic> body => dmxList<Product>(body, 'search', Product.fromJson),          _ => Err<List<Product>, DecodeError>(DecodeError('search', 'List<Product>', response.body)) }) {
            Ok(value: final value) => Ok(value),
            Err(error: final e) => Err(ApiDecodeFailure(e)),
          },
      };

  @override
  Future<Result<Placed, ApiError>> placeOrder(Draft draft) async =>
      switch (await transport.send(DmxRequest(
        method: 'POST',
        url: Uri.parse('$baseUrl/orders'),
        headers: <String, String>{
          ...defaultHeaders,
          'content-type': 'application/json',
          ...headers,
        },
        body: draft.toJson(),
      ))) {
        Err(error: final e) => Err(ApiTransportFailure(e)),
        Ok(value: final response) when !response.isSuccess =>
          Err(ApiStatusFailure(response.status, response.body)),
        Ok(value: final response) => switch (Placed.fromJson(response.body, 'placeOrder')) {
            Ok(value: final value) => Ok(value),
            Err(error: final e) => Err(ApiDecodeFailure(e)),
          },
      };

  @override
  Future<Result<Cancelled, ApiError>> refund(String orderId, RefundReason reason) async =>
      switch (await transport.send(DmxRequest(
        method: 'POST',
        url: Uri.parse('$baseUrl/orders/$orderId/refund').replace(queryParameters: dmxQuery(<String, String?>{'reason': reason.toJson()})),
        headers: <String, String>{
          ...defaultHeaders,
          ...headers,
        },
      ))) {
        Err(error: final e) => Err(ApiTransportFailure(e)),
        Ok(value: final response) when !response.isSuccess =>
          Err(ApiStatusFailure(response.status, response.body)),
        Ok(value: final response) => switch (Cancelled.fromJson(response.body, 'refund')) {
            Ok(value: final value) => Ok(value),
            Err(error: final e) => Err(ApiDecodeFailure(e)),
          },
      };

  @override
  Future<Result<void, ApiError>> abandonCart(String cartId) async =>
      switch (await transport.send(DmxRequest(
        method: 'DELETE',
        url: Uri.parse('$baseUrl/carts/$cartId'),
        headers: <String, String>{
          ...defaultHeaders,
          ...headers,
        },
      ))) {
        Err(error: final e) => Err(ApiTransportFailure(e)),
        Ok(value: final response) when !response.isSuccess =>
          Err(ApiStatusFailure(response.status, response.body)),
        Ok() => Ok<void, ApiError>(null),
      };
  //#endregion
}
