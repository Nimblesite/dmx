/// [catalogue.fake]: fixtures that are the same tomorrow.
library;

import 'package:dmx/dmx.dart';
import 'package:dmx_storefront_example/payments.dart';
import 'package:dmx_storefront_example/testing.dart';
import 'package:test/test.dart';

void main() {
  group('determinism', () {
    test('the same seed builds the same fixture, every time', () {
      expect(Customer.fake(), Customer.fake());
      expect(Customer.fake(seed: 3), Customer.fake(seed: 3));
    });

    test('a different seed builds a different fixture', () {
      expect(Customer.fake(seed: 1), isNot(Customer.fake(seed: 2)));
    });

    test('there is no randomness to be flaky about', () {
      final hashes = List<int>.generate(20, (_) => Customer.fake().hashCode);
      expect(hashes.toSet(), hasLength(1));
    });
  });

  group('field rules', () {
    test('strings are named after their field, so a failure reads clearly', () {
      expect(Customer.fake(seed: 0).id, 'id-0');
      expect(Customer.fake(seed: 0).email, 'email-1@example.test');
    });

    test('numbers walk with the seed', () {
      expect(Customer.fake(seed: 10).loyaltyPoints, 12);
    });

    test('dates step off a fixed epoch, not off the clock', () {
      expect(Customer.fake(seed: 0).joinedAt, DateTime.utc(2024, 1, 4));
    });

    test('enums cycle through their own constants', () {
      final methods = List<PaymentMethod>.generate(
        PaymentMethod.values.length,
        (index) => Customer.fake(seed: index).preferredMethod,
      );
      expect(methods.toSet(), PaymentMethod.values.toSet());
    });

    test("a nested @dmx('fake') type builds its own fixture", () {
      expect(Customer.fake(seed: 0).address, Address.fake(seed: 6));
    });

    test('nullable fields default to null — simplest valid, not fullest', () {
      expect(Customer.fake().referredBy, isNull);
    });
  });

  group('overrides', () {
    test('a test states only what the test is about', () {
      final customer = Customer.fake(isVip: true, loyaltyPoints: 9001);
      expect(customer.isVip, isTrue);
      expect(customer.loyaltyPoints, 9001);
      expect(customer.id, Customer.fake().id);
    });

    test('an override reaches a nullable field too', () {
      expect(Customer.fake(referredBy: 'friend').referredBy, 'friend');
    });
  });

  group('collections', () {
    test('fakes builds distinct fixtures', () {
      final customers = Customer.fakes(5);
      expect(customers, hasLength(5));
      expect(customers.toSet(), hasLength(5));
    });

    test('fakes is itself deterministic', () {
      expect(Customer.fakes(3), Customer.fakes(3));
    });
  });

  group('composition with the codec', () {
    test('a fixture round-trips through its own model codec', () {
      final customer = Customer.fake();
      expect(
        Customer.fromJson(customer.toJson()),
        Ok<Customer, DecodeError>(customer),
      );
    });

    test('fakeJson exercises a decoder against a guaranteed-valid payload', () {
      expect(
        Customer.fromJson(Customer.fakeJson(seed: 5)),
        Ok<Customer, DecodeError>(Customer.fake(seed: 5)),
      );
    });
  });
}
