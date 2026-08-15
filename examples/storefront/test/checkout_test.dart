/// [catalogue.validate]: every violation at once, as data.
library;

import 'package:dmx/dmx.dart';
import 'package:dmx_storefront_example/checkout.dart';
import 'package:test/test.dart';

const valid = CheckoutForm(
  email: 'buyer@example.test',
  postcode: 'SW1A1AA',
  quantity: 2,
  acceptsTerms: true,
);

void main() {
  group('CheckoutForm', () {
    test('a good form has no violations', () {
      expect(valid.violations, isEmpty);
      expect(valid.isValid, isTrue);
      expect(valid.validate(), Ok<CheckoutForm, List<Violation>>(valid));
    });

    test('violations accumulate — all of them, not the first', () {
      const form = CheckoutForm(
        email: 'nope',
        postcode: 'x',
        quantity: 0,
        acceptsTerms: false,
      );
      expect(form.violations, hasLength(4));
      expect(
        form.violations.map((violation) => violation.field),
        <String>['email', 'postcode', 'quantity', 'accepts_terms'],
      );
    });

    test('two rules on one field can both fire', () {
      const form = CheckoutForm(
        email: '',
        postcode: 'SW1A1AA',
        quantity: 1,
        acceptsTerms: true,
      );
      expect(form.messagesFor('email'), <String>[
        'must not be empty',
        'must look like an email address',
      ]);
    });

    test('violations are named with the wire key, ready for a form control',
        () {
      const form = CheckoutForm(
        email: 'buyer@example.test',
        postcode: 'SW1A1AA',
        quantity: 1,
        acceptsTerms: false,
      );
      expect(form.messagesFor('accepts_terms'),
          <String>['the terms must be accepted']);
    });

    test('an absent nullable field is not a violation', () {
      expect(valid.giftMessage, isNull);
      expect(valid.messagesFor('gift_message'), isEmpty);
    });

    test('a present nullable field is checked', () {
      final form = valid.copyWith(giftMessage: DmxTo('x' * 201));
      expect(form.messagesFor('gift_message'),
          <String>['must be at most 200 characters']);
    });

    test('a pattern rule reports a format problem, not a crash', () {
      final form = valid.copyWith(discountCode: const DmxTo('nope!'));
      expect(
          form.messagesFor('discount_code'), <String>['has the wrong format']);
      expect(
          form.copyWith(discountCode: const DmxTo('SPRING24')).isValid, isTrue);
    });

    test('validate returns the violations as the error channel', () {
      const form = CheckoutForm(
        email: 'nope',
        postcode: 'SW1A1AA',
        quantity: 1,
        acceptsTerms: true,
      );
      expect(
        form.validate(),
        isA<Err<CheckoutForm, List<Violation>>>()
            .having((e) => e.error, 'error', hasLength(1)),
      );
    });

    test('validation composes with the codec on the same class', () {
      final json = valid.toJson();
      expect(
        CheckoutForm.fromJson(json),
        isA<Ok<CheckoutForm, DecodeError>>()
            .having((r) => r.value.isValid, 'isValid', isTrue),
      );
    });
  });

  group('ShippingAddress', () {
    test('a custom message replaces both halves of a length rule', () {
      const address = ShippingAddress(
        line1: '1 Test Street',
        city: 'London',
        country: 'GBR',
      );
      expect(address.messagesFor('country'),
          <String>['must be an ISO country code']);
    });

    test('a good address validates', () {
      const address = ShippingAddress(
        line1: '1 Test Street',
        city: 'London',
        country: 'GB',
      );
      expect(address.isValid, isTrue);
    });
  });
}
