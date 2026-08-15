/// [catalogue.union]: discriminated decode, exhaustive dispatch, narrowing.
library;

import 'package:dmx/dmx.dart';
import 'package:dmx_storefront_example/catalog.dart';
import 'package:dmx_storefront_example/orders.dart';
import 'package:dmx_storefront_example/payments.dart';
import 'package:test/test.dart';

const money = Money(amount: 4200, currency: 'GBP');

final placedJson = <String, dynamic>{
  'type': 'placed',
  'order_id': 'o-1',
  'placed_at': '2024-05-06T12:00:00.000Z',
  'method': 'apple_pay',
  'total': <String, dynamic>{'amount': 4200, 'currency': 'GBP'},
};

void main() {
  group('decoding', () {
    test('dispatches on the discriminator to the right variant', () {
      expect(
        OrderState.fromJson(placedJson),
        isA<Ok<OrderState, DecodeError>>().having(
          (r) => r.value,
          'value',
          isA<Placed>()
              .having((p) => p.method, 'method', PaymentMethod.applePay),
        ),
      );
    });

    test('an unknown tag fails at the discriminator, naming it', () {
      final json = <String, dynamic>{...placedJson, 'type': 'refunded'};
      expect(
        OrderState.fromJson(json),
        isA<Err<OrderState, DecodeError>>()
            .having((e) => e.error.path, 'path', 'OrderState.type')
            .having((e) => e.error.actual, 'actual', 'refunded'),
      );
    });

    test('a missing tag fails at the union, not at a variant', () {
      final json = <String, dynamic>{...placedJson}..remove('type');
      expect(
        OrderState.fromJson(json),
        isA<Err<OrderState, DecodeError>>()
            .having((e) => e.error.expected, 'expected', 'OrderState'),
      );
    });

    test('the failure path records which variant was being decoded', () {
      final json = <String, dynamic>{...placedJson, 'placed_at': 'soon'};
      expect(
        OrderState.fromJson(json),
        isA<Err<OrderState, DecodeError>>().having(
          (e) => e.error.path,
          'path',
          'OrderState(placed).placed_at',
        ),
      );
    });

    test('every variant round-trips through the union codec', () {
      final states = <OrderState>[
        const Draft(cartId: 'c-1', lines: <OrderLine>[
          OrderLine(sku: 'kettle', quantity: 1, unitPrice: money),
        ]),
        Placed(
          orderId: 'o-1',
          placedAt: DateTime.utc(2024, 5, 6, 12),
          method: PaymentMethod.applePay,
          total: money,
        ),
        const Shipped(
          orderId: 'o-1',
          carrier: 'Royal Mail',
          trackingNumber: 'RM1',
        ),
        const Cancelled(orderId: 'o-1', reason: RefundReason.fraudulent),
      ];
      for (final state in states) {
        expect(
          OrderState.fromJson(state.toJson()),
          Ok<OrderState, DecodeError>(state),
          reason: '$state did not survive the round trip',
        );
      }
    });

    test('the tag is written on the way out as well as read on the way in', () {
      const state = Shipped(
        orderId: 'o-1',
        carrier: 'Royal Mail',
        trackingNumber: 'RM1',
      );
      expect(state.toJson()['type'], 'shipped');
    });
  });

  group('dispatch', () {
    final OrderState state = Placed(
      orderId: 'o-1',
      placedAt: DateTime.utc(2024, 5, 6, 12),
      method: PaymentMethod.card,
      total: money,
    );

    test('when reaches exactly one arm', () {
      expect(
        state.when(
          draft: (value) => 'draft',
          placed: (value) => 'placed ${value.orderId}',
          shipped: (value) => 'shipped',
          cancelled: (value) => 'cancelled',
        ),
        'placed o-1',
      );
    });

    test('maybeWhen falls back when the arm is absent', () {
      expect(state.maybeWhen(draft: (value) => 'draft', orElse: () => 'other'),
          'other');
    });

    test('maybeWhen does not confuse a null result with an absent arm', () {
      // `?.call() ?? orElse()` would have taken the fallback here.
      expect(
        state.maybeWhen<String?>(
            placed: (value) => null, orElse: () => 'other'),
        isNull,
      );
    });

    test('narrowing accessors return the variant or null, never a cast', () {
      expect(state.asPlaced, isA<Placed>());
      expect(state.asDraft, isNull);
      expect(state.isPlaced, isTrue);
    });
  });
}
