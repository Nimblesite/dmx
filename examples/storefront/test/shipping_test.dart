// Proves the two files generated from models/shipping.td [typediagram].
//
// Nothing here is generated. The point of the suite is that a definition
// written once as a typeDiagram file, with no Dart source of truth and no `@dmx`
// annotation anywhere, produces Dart you can actually construct, match on, and
// index — and that both outputs agree, because both are functions of the same
// definition.

import 'package:dmx/dmx.dart';
import 'package:dmx_storefront_example/shipping.dart';
import 'package:dmx_storefront_example/shipping_wire.dart';
import 'package:test/test.dart';

/// A parcel built at run time, so two of them are separate objects — which is
/// what makes an equality test about equality rather than about identity.
Parcel parcelNamed(String id) =>
    Parcel(id: id, weightG: 1200, labels: <String>['fragile', 'up']);

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

  group('value semantics', () {
    test('two values built from the same fields are equal and hash alike', () {
      final one = parcelNamed('b0a1');
      final two = parcelNamed('b0a1');

      expect(identical(one, two), isFalse);
      expect(one, two, reason: 'a diagram declares values, not identities');
      expect(one.hashCode, two.hashCode);
      expect(<Parcel>{one, two}, hasLength(1));
      expect(<Parcel>{one, parcelNamed('b0a2')}, hasLength(2));
    });

    test('a list field compares by content, not by reference', () {
      const one = Parcel(id: 'b0a1', weightG: 1, labels: <String>['a']);
      const two = Parcel(id: 'b0a1', weightG: 1, labels: <String>['b']);

      expect(one, isNot(two));
    });

    test('copyWith replaces what it is given and keeps the rest', () {
      const parcel = Parcel(
          id: 'b0a1', weightG: 1200, insured: '19.99', labels: <String>[]);

      expect(parcel.copyWith(weightG: 30).weightG, 30);
      expect(parcel.copyWith(weightG: 30).id, 'b0a1');
      expect(parcel.copyWith().insured, '19.99',
          reason: 'omitting a nullable field keeps it');
      expect(parcel.copyWith(insured: const DmxTo(null)).insured, isNull,
          reason: 'clearing one is a different call from omitting it');
    });

    test('toString names the class and every field that carries a value', () {
      const parcel = Parcel(id: 'b0a1', weightG: 12, labels: <String>[]);

      expect(parcel.toString(),
          'Parcel(id: b0a1, weightG: 12, insured: null, labels: [])');
    });
  });

  group('json, which lives beside the class rather than in it', () {
    test('a record round-trips through its extension', () {
      const parcel = Parcel(
          id: 'b0a1', weightG: 1200, labels: <String>['fragile', 'up']);

      final json = parcel.toJson();
      expect(json, <String, Object?>{
        'id': 'b0a1',
        'weightG': 1200,
        'insured': null,
        'labels': <String>['fragile', 'up'],
      });
      expect(ParcelJson.fromJson(json), Ok<Parcel, DecodeError>(parcel));
    });

    test('a nested record and a list of union cases decode too', () {
      final shipment = Shipment(
        parcel: const Parcel(id: 'c3', weightG: 10, labels: <String>[]),
        legs: <Leg>[Pickup(at: DateTime.utc(2026, 8, 19, 9))],
        tracking: 'NF-0001',
      );

      expect(ShipmentJson.fromJson(shipment.toJson()),
          Ok<Shipment, DecodeError>(shipment));
    });

    test('a union tags itself on the way out and reads the tag back', () {
      final leg = Transit(carrier: 'Nimble Freight', etaHours: 30);

      expect(leg.toJson(),
          <String, Object?>{'carrier': 'Nimble Freight', 'etaHours': 30});
      expect(LegJson.fromJson(<String, Object?>{
        'type': 'transit',
        'carrier': 'Nimble Freight',
        'etaHours': 30,
      }), Ok<Leg, DecodeError>(leg));
    });

    test('a bad payload is an error value, never an exception', () {
      final decoded = ParcelJson.fromJson(<String, Object?>{'id': 7});

      expect(decoded, isA<Err<Parcel, DecodeError>>());
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
