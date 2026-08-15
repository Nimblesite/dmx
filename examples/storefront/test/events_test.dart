/// [catalogue.events]: analytics that cannot be misspelled.
library;

import 'package:dmx_storefront_example/events.dart';
import 'package:dmx_storefront_example/payments.dart';
import 'package:test/test.dart';

/// A real sink, not a mock: it keeps what it was given.
class RecordingSink implements AnalyticsSink {
  final List<(String, Map<String, Object?>)> recorded =
      <(String, Map<String, Object?>)>[];

  @override
  void record(String name, Map<String, Object?> parameters) =>
      recorded.add((name, parameters));
}

void main() {
  test('the event name lives in exactly one place', () {
    expect(ProductViewed.eventName, 'product_viewed');
    expect(
      const ProductViewed(productId: 'p', priceCents: 1, currency: 'GBP').name,
      ProductViewed.eventName,
    );
  });

  test('parameters are flat and snake-cased', () {
    expect(
      const CartItemAdded(sku: 'k-1', quantity: 2, cartValueCents: 500)
          .parameters,
      <String, Object?>{
        'sku': 'k-1',
        'quantity': 2,
        'cart_value_cents': 500,
      },
    );
  });

  test('an absent optional parameter is absent, not null', () {
    // A null in an analytics payload becomes a real bucket in most dashboards.
    const event = ProductViewed(productId: 'p', priceCents: 1, currency: 'GBP');
    expect(event.parameters.containsKey('referrer'), isFalse);
  });

  test('a present optional parameter is included', () {
    const event = ProductViewed(
      productId: 'p',
      priceCents: 1,
      currency: 'GBP',
      referrer: 'newsletter',
    );
    expect(event.parameters['referrer'], 'newsletter');
  });

  test('an int optional parameter behaves the same way', () {
    const absent = CheckoutFailed(stage: 'payment', reason: 'declined');
    const present = CheckoutFailed(
      stage: 'payment',
      reason: 'declined',
      httpStatus: 402,
    );
    expect(absent.parameters.containsKey('http_status'), isFalse);
    expect(present.parameters['http_status'], 402);
  });

  test('an enum parameter uses the same wire name the API uses', () {
    const event = CheckoutCompleted(
      orderId: 'o-1',
      method: PaymentMethod.bankTransfer,
      totalCents: 4200,
      lineCount: 2,
      usedDiscount: false,
    );
    expect(event.parameters['method'], 'ach_transfer');
    expect(event.parameters['method'], PaymentMethod.bankTransfer.wire);
  });

  test('every event satisfies the hand-written interface', () {
    final events = <AnalyticsEvent>[
      const ProductViewed(productId: 'p', priceCents: 1, currency: 'GBP'),
      const CartItemAdded(sku: 'k', quantity: 1, cartValueCents: 1),
      const CheckoutCompleted(
        orderId: 'o',
        method: PaymentMethod.card,
        totalCents: 1,
        lineCount: 1,
        usedDiscount: true,
      ),
      const CheckoutFailed(stage: 'payment', reason: 'declined'),
    ];
    final sink = RecordingSink();
    for (final event in events) {
      sink.log(event);
    }
    expect(sink.recorded.map((entry) => entry.$1), <String>[
      'product_viewed',
      'cart_item_added',
      'checkout_completed',
      'checkout_failed',
    ]);
  });

  test('one call site dispatches every event', () {
    final sink = RecordingSink();
    sink.log(const CartItemAdded(sku: 'k', quantity: 3, cartValueCents: 900));
    expect(sink.recorded.single.$2['quantity'], 3);
  });
}
