/// [catalogue.enum]: wire names, labels, and a codec that never throws.
library;

import 'package:dmx/dmx.dart';
import 'package:dmx_storefront_example/payments.dart';
import 'package:test/test.dart';

void main() {
  group('PaymentMethod', () {
    test('encodes with the renamed wire name, not the Dart identifier', () {
      expect(PaymentMethod.applePay.toJson(), 'apple_pay');
      expect(PaymentMethod.card.toJson(), 'card');
    });

    test("an explicit @dmx('value', {'wire': ...}) beats the fieldRename policy", () {
      expect(PaymentMethod.bankTransfer.wire, 'ach_transfer');
      expect(PaymentMethod.bankTransfer.wire, isNot('bank_transfer'));
    });

    test("labels come from @dmx('value', {'label': ...}) or the humanised identifier", () {
      expect(PaymentMethod.bankTransfer.label, 'Bank transfer');
      expect(PaymentMethod.googlePay.label, 'Google pay');
    });

    test('round-trips every constant', () {
      for (final method in PaymentMethod.values) {
        expect(PaymentMethod.tryParse(method.wire), method);
        expect(
          PaymentMethod.fromJson(method.toJson()),
          isA<Ok<PaymentMethod, DecodeError>>(),
        );
      }
    });

    test('an unknown wire name is a decode failure, not an exception', () {
      final result = PaymentMethod.fromJson('paypal', 'Order.method');
      expect(
        result,
        isA<Err<PaymentMethod, DecodeError>>()
            .having((e) => e.error.path, 'path', 'Order.method')
            .having((e) => e.error.expected, 'expected', 'PaymentMethod'),
      );
    });

    test('a non-string is a decode failure carrying the offending value', () {
      expect(
        PaymentMethod.fromJson(7),
        isA<Err<PaymentMethod, DecodeError>>()
            .having((e) => e.error.actual, 'actual', 7),
      );
    });

    test('the narrowing getters agree with the constant', () {
      expect(PaymentMethod.card.isCard, isTrue);
      expect(PaymentMethod.card.isGiftCard, isFalse);
    });
  });

  group('RefundReason', () {
    test('screaming_snake is the wire policy for this enum', () {
      expect(RefundReason.itemNotReceived.wire, 'ITEM_NOT_RECEIVED');
    });

    test('an unknown value falls back instead of failing', () {
      // The whole point of `unknown:`: a constant added upstream after this
      // build shipped must not take the payload down.
      expect(
        RefundReason.fromJson('INVENTED_UPSTREAM'),
        Ok<RefundReason, DecodeError>(RefundReason.other),
      );
    });

    test('the fallback does not swallow a wrong *type*', () {
      expect(
        RefundReason.fromJson(const <String>[]),
        isA<Err<RefundReason, DecodeError>>(),
      );
    });
  });

  group('Fulfilment', () {
    test('hand-written members survive regeneration untouched', () {
      expect(Fulfilment.delivered.isTerminal, isTrue);
      expect(Fulfilment.packed.isTerminal, isFalse);
    });

    test('generated members sit beside them', () {
      expect(Fulfilment.packed.label, 'Packed');
      expect(Fulfilment.tryParse('shipped'), Fulfilment.shipped);
      expect(Fulfilment.tryParse('nope'), isNull);
    });
  });
}
