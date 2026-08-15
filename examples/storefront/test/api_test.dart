/// [catalogue.rest]: the generated client, driven by a real transport.
///
/// There is no mock framework here and there is no HTTP server either. The
/// transport is the seam, so a handful of lines of ordinary Dart exercises the
/// generated code exactly as it runs in production.
library;

import 'package:dmx/dmx.dart';
import 'package:dmx_storefront_example/api.dart';
import 'package:dmx_storefront_example/catalog.dart';
import 'package:dmx_storefront_example/orders.dart';
import 'package:dmx_storefront_example/payments.dart';
import 'package:test/test.dart';

/// Answers from a script, and records what it was asked.
class ScriptedTransport implements DmxTransport {
  ScriptedTransport(this.responses);

  final List<Result<DmxResponse, TransportError>> responses;
  final List<DmxRequest> sent = <DmxRequest>[];

  @override
  Future<Result<DmxResponse, TransportError>> send(DmxRequest request) async {
    sent.add(request);
    return responses.isEmpty
        ? const Err(TransportError('nothing scripted'))
        : responses.removeAt(0);
  }
}

Map<String, dynamic> productJson(String id) => <String, dynamic>{
      'id': id,
      'title': 'Kettle',
      'variants': <dynamic>[],
      'tags': <dynamic>[],
      'accepted_methods': <dynamic>['card'],
      'published_at': '2024-03-01T00:00:00.000Z',
    };

void main() {
  test('builds the URL from the base and the path template', () async {
    final transport = ScriptedTransport(<Result<DmxResponse, TransportError>>[
      Ok(DmxResponse(status: 200, body: productJson('kettle'))),
    ]);
    await StorefrontClient(transport).product('kettle');
    expect(
      transport.sent.single.url.toString(),
      'https://api.storefront.example/v1/products/kettle',
    );
    expect(transport.sent.single.method, 'GET');
  });

  test('decodes a successful response with the model codec', () async {
    final transport = ScriptedTransport(<Result<DmxResponse, TransportError>>[
      Ok(DmxResponse(status: 200, body: productJson('kettle'))),
    ]);
    expect(
      await StorefrontClient(transport).product('kettle'),
      isA<Ok<Product, ApiError>>().having((r) => r.value.id, 'id', 'kettle'),
    );
  });

  test('a transport failure is classified, not thrown', () async {
    final transport = ScriptedTransport(<Result<DmxResponse, TransportError>>[
      const Err(TransportError('connection reset')),
    ]);
    expect(
      await StorefrontClient(transport).product('kettle'),
      isA<Err<Product, ApiError>>()
          .having((e) => e.error, 'error', isA<ApiTransportFailure>()),
    );
  });

  test('a non-2xx response keeps the status and the body', () async {
    final transport = ScriptedTransport(<Result<DmxResponse, TransportError>>[
      const Ok(DmxResponse(status: 404, body: <String, dynamic>{})),
    ]);
    expect(
      await StorefrontClient(transport).product('kettle'),
      isA<Err<Product, ApiError>>().having(
        (e) => e.error,
        'error',
        isA<ApiStatusFailure>().having((f) => f.status, 'status', 404),
      ),
    );
  });

  test('a 200 with a malformed body is a decode failure, not a status one',
      () async {
    final transport = ScriptedTransport(<Result<DmxResponse, TransportError>>[
      const Ok(DmxResponse(status: 200, body: <String, dynamic>{'id': 1})),
    ]);
    expect(
      await StorefrontClient(transport).product('kettle'),
      isA<Err<Product, ApiError>>()
          .having((e) => e.error, 'error', isA<ApiDecodeFailure>()),
    );
  });

  test('query parameters come from the annotated arguments', () async {
    final transport = ScriptedTransport(<Result<DmxResponse, TransportError>>[
      const Ok(DmxResponse(status: 200, body: <dynamic>[])),
    ]);
    await StorefrontClient(transport).search(q: 'kettle', page: 2);
    expect(transport.sent.single.url.queryParameters,
        <String, String>{'q': 'kettle', 'page': '2'});
  });

  test('a list response decodes element by element', () async {
    final transport = ScriptedTransport(<Result<DmxResponse, TransportError>>[
      Ok(DmxResponse(
        status: 200,
        body: <dynamic>[productJson('a'), productJson('b')],
      )),
    ]);
    expect(
      await StorefrontClient(transport).search(q: 'kettle'),
      isA<Ok<List<Product>, ApiError>>()
          .having((r) => r.value.map((p) => p.id), 'ids', <String>['a', 'b']),
    );
  });

  test('a body argument is encoded and the content type set', () async {
    final transport = ScriptedTransport(<Result<DmxResponse, TransportError>>[
      Ok(DmxResponse(status: 201, body: <String, dynamic>{
        'type': 'placed',
        'order_id': 'o-1',
        'placed_at': '2024-05-06T12:00:00.000Z',
        'method': 'card',
        'total': <String, dynamic>{'amount': 1, 'currency': 'GBP'},
      })),
    ]);
    const draft = Draft(cartId: 'c-1', lines: <OrderLine>[]);
    final result = await StorefrontClient(transport).placeOrder(draft);
    expect(transport.sent.single.body, draft.toJson());
    expect(transport.sent.single.headers['content-type'], 'application/json');
    expect(result, isA<Ok<Placed, ApiError>>());
  });

  test('an enum argument uses its wire name', () async {
    final transport = ScriptedTransport(<Result<DmxResponse, TransportError>>[
      const Ok(DmxResponse(status: 500)),
    ]);
    await StorefrontClient(transport).refund('o-1', RefundReason.fraudulent);
    expect(
      transport.sent.single.url.queryParameters['reason'],
      'FRAUDULENT',
    );
  });

  test('a void endpoint succeeds on status alone', () async {
    final transport = ScriptedTransport(<Result<DmxResponse, TransportError>>[
      const Ok(DmxResponse(status: 204)),
    ]);
    expect(
      await StorefrontClient(transport).abandonCart('c-1'),
      isA<Ok<void, ApiError>>(),
    );
  });

  test('caller headers are merged over the defaults', () async {
    final transport = ScriptedTransport(<Result<DmxResponse, TransportError>>[
      Ok(DmxResponse(status: 200, body: productJson('kettle'))),
    ]);
    await StorefrontClient(
      transport,
      headers: const <String, String>{'authorization': 'Bearer t'},
    ).product('kettle');
    expect(transport.sent.single.headers, <String, String>{
      'accept': 'application/json',
      'authorization': 'Bearer t',
    });
  });
}
