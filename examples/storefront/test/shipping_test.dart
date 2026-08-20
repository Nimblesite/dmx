// Proves the two files generated from docs/shipping.dmx.md [typediagram].
//
// Nothing here is generated. The point of the suite is that a definition
// written once in Markdown, with no Dart source of truth and no `@dmx`
// annotation anywhere, produces Dart you can actually construct, match on, and
// index — and that both outputs agree, because both are functions of the same
// definition.

import 'package:dmx_storefront_example/shipping.dart';
import 'package:dmx_storefront_example/shipping_wire.dart';
import 'package:test/test.dart';

/// A leg description that proves the switch is exhaustive: no default arm, no
/// cast, no null assertion — the sealed class is what makes that possible.
String describe(Leg leg) => switch (leg) {
      Pickup(at: final at) => 'picked up at ${at.toIso8601String()}',
      Transit(carrier: final carrier, etaHours: final hours) =>
        '$carrier, $hours hours out',
      Delivered(signedBy: final signedBy) when signedBy == null =>
        'delivered, unsigned',
      Delivered(signedBy: final signedBy) => 'delivered, signed by $signedBy',
    };

void main() {
  group('records', () {
    test('every field arrives with the Dart type the diagram implies', () {
      const parcel = Parcel(
        id: 'b0a1',
        weightG: 1200,
        labels: <String>['fragile', 'this way up'],
      );

      expect(parcel.id, 'b0a1');
      expect(parcel.weightG, 1200);
      expect(parcel.labels, <String>['fragile', 'this way up']);
      expect(parcel.insured, isNull,
          reason: 'Option<Decimal> is a nullable Dart field, so it defaults');
    });

    test('an optional field is optional and a required one is required', () {
      const insured = Parcel(
        id: 'b0a2',
        weightG: 40,
        insured: '19.99',
        labels: <String>[],
      );
      expect(insured.insured, '19.99');
      expect(insured.labels, isEmpty);
    });

    test('a record composes with the other declarations', () {
      final shipment = Shipment(
        parcel: const Parcel(id: 'c3', weightG: 10, labels: <String>[]),
        legs: <Leg>[
          Pickup(at: DateTime.utc(2026, 8, 19, 9)),
          const Transit(carrier: 'Nimble Freight', etaHours: 30),
        ],
        tracking: 'NF-0001',
      );

      expect(shipment.parcel.id, 'c3');
      expect(shipment.legs, hasLength(2));
      expect(shipment.tracking, 'NF-0001');
      expect(shipment.tracking, isA<TrackingNumber>(),
          reason: 'the alias is a typedef, so it is the same type');
    });
  });

  group('the union', () {
    test('every variant is a subtype of the sealed base', () {
      final legs = <Leg>[
        Pickup(at: DateTime.utc(2026, 8, 19, 9)),
        const Transit(carrier: 'Nimble Freight', etaHours: 30),
        Delivered(at: DateTime.utc(2026, 8, 21, 14), signedBy: 'R. Patel'),
      ];
      expect(legs.whereType<Pickup>(), hasLength(1));
      expect(legs.whereType<Transit>(), hasLength(1));
      expect(legs.whereType<Delivered>(), hasLength(1));
    });

    test('a switch over it is exhaustive without a default arm', () {
      expect(describe(Pickup(at: DateTime.utc(2026, 8, 19, 9))),
          'picked up at 2026-08-19T09:00:00.000Z');
      expect(describe(const Transit(carrier: 'Nimble Freight', etaHours: 30)),
          'Nimble Freight, 30 hours out');
      expect(describe(Delivered(at: DateTime.utc(2026, 8, 21), signedBy: null)),
          'delivered, unsigned');
      expect(
          describe(
              Delivered(at: DateTime.utc(2026, 8, 21), signedBy: 'R. Patel')),
          'delivered, signed by R. Patel');
    });
  });

  group('the wire-name table', () {
    test('it carries every record and every variant', () {
      expect(
        shippingWireNames.keys,
        containsAll(<String>[
          'Parcel',
          'Leg.Pickup',
          'Leg.Transit',
          'Leg.Delivered',
          'Shipment',
        ]),
      );
    });

    test('camel case becomes snake case, and single words do not change', () {
      expect(shippingWireNames['Parcel']!['weightG'], 'weight_g');
      expect(shippingWireNames['Parcel']!['id'], 'id');
      expect(shippingWireNames['Leg.Transit']!['etaHours'], 'eta_hours');
      expect(shippingWireNames['Leg.Delivered']!['signedBy'], 'signed_by');
    });

    test('the declaration list is the diagram order, aliases included', () {
      expect(shippingDeclarations,
          <String>['Parcel', 'Leg', 'TrackingNumber', 'Shipment']);
    });

    test('both generated files describe the same declarations', () {
      final fromTable = shippingWireNames.keys
          .map((key) => key.split('.').first)
          .toSet();
      expect(
        fromTable,
        <String>{'Parcel', 'Leg', 'Shipment'},
        reason: 'one definition, two outputs, and no way for them to disagree',
      );
      expect(
        shippingDeclarations.toSet().containsAll(fromTable),
        isTrue,
      );
    });
  });
}
