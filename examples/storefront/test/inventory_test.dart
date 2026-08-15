/// [catalogue.diff]: what changed, as data.
library;

import 'package:dmx/dmx.dart';
import 'package:dmx_storefront_example/inventory.dart';
import 'package:test/test.dart';

const before = StockLevel(
  sku: 'kettle-black',
  onHand: 12,
  reserved: 2,
  reorderPoint: 5,
  locations: <String, int>{'LON': 8, 'MAN': 4},
);

void main() {
  group('diff', () {
    test('identical values produce nothing', () {
      expect(before.diff(before), isEmpty);
      expect(before.differsFrom(before), isFalse);
    });

    test('reports the field, the old value, and the new one', () {
      final after = before.copyWith(onHand: 9);
      expect(before.diff(after), <DmxChange>[
        const DmxChange('on_hand', 12, 9),
      ]);
    });

    test('reports several changes at once, in field order', () {
      final after = before.copyWith(onHand: 9, reorderPoint: 3);
      expect(
        before.changedFields(after),
        <String>['on_hand', 'reorder_point'],
      );
    });

    test('field names are the wire names, ready for an audit row', () {
      final after = before.copyWith(reorderPoint: 1);
      expect(before.diff(after).single.field, 'reorder_point');
    });

    test('collections compare by content, so a rebuild is not a change', () {
      final after = before.copyWith(
        locations: <String, int>{...before.locations},
      );
      expect(before.diff(after), isEmpty);
    });

    test('a genuine collection change is reported', () {
      final after = before.copyWith(
        locations: const <String, int>{'LON': 8, 'MAN': 3},
      );
      expect(before.diff(after).single.field, 'locations');
    });

    test('a nullable field going from null to a value is a change', () {
      final after = before.copyWith(discontinuedAt: DmxTo(DateTime.utc(2024)));
      expect(before.diff(after), <DmxChange>[
        DmxChange('discontinued_at', null, DateTime.utc(2024)),
      ]);
    });

    test('and back to null is a change too', () {
      final discontinued =
          before.copyWith(discontinuedAt: DmxTo(DateTime.utc(2024)));
      expect(
        discontinued.diff(before).single,
        DmxChange('discontinued_at', DateTime.utc(2024), null),
      );
    });

    test('diff agrees with ==', () {
      final after = before.copyWith(reserved: 4);
      expect(before == after, isFalse);
      expect(before.diff(after), isNotEmpty);

      final same = before.copyWith();
      expect(before == same, isTrue);
      expect(before.diff(same), isEmpty);
    });
  });

  group('composition', () {
    test('the class is still a model: it round-trips', () {
      expect(
        StockLevel.fromJson(before.toJson()),
        Ok<StockLevel, DecodeError>(before),
      );
    });

    test('hand-written members are untouched by either macro', () {
      expect(before.available, 10);
      expect(before.needsReorder, isFalse);
      expect(before.copyWith(onHand: 6).needsReorder, isTrue);
    });
  });

  group('a field named after a Dart operator keyword', () {
    test('is generated with its own name, not mangled', () {
      final adjustment = StockAdjustment(
        sku: 'kettle-black',
        delta: -3,
        reason: 'damaged',
        at: DateTime.utc(2024, 7, 1),
        operator: 'ada',
      );
      expect(adjustment.operator, 'ada');
      expect(adjustment.toJson()['operator'], 'ada');
      expect(
        StockAdjustment.fromJson(adjustment.toJson()),
        Ok<StockAdjustment, DecodeError>(adjustment),
      );
    });

    test('and diffs like anything else', () {
      final adjustment = StockAdjustment(
        sku: 'kettle-black',
        delta: -3,
        reason: 'damaged',
        at: DateTime.utc(2024, 7, 1),
        operator: 'ada',
      );
      expect(
        adjustment.diff(adjustment.copyWith(operator: 'grace')).single,
        const DmxChange('operator', 'ada', 'grace'),
      );
    });
  });
}
