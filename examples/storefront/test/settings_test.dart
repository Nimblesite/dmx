/// [catalogue.prefs]: typed settings over an untyped store, that never fail.
library;

import 'package:dmx_storefront_example/settings.dart';
import 'package:test/test.dart';

/// The real interface, backed by a map. No mock framework involved.
class MemoryStore implements KeyValueStore {
  MemoryStore([Map<String, Object?>? initial])
      : values = <String, Object?>{...?initial};

  final Map<String, Object?> values;

  @override
  Object? read(String key) => values[key];

  @override
  void write(String key, Object? value) => values[key] = value;

  @override
  void remove(String key) => values.remove(key);
}

void main() {
  group('keys', () {
    test('every key is namespaced, so nothing collides', () {
      for (final key in Settings.keys) {
        expect(key, startsWith('${Settings.namespace}.'));
      }
    });

    test('keys are snake-cased regardless of the Dart identifier', () {
      expect(Settings.launchCountKey, 'storefront.launch_count');
    });

    test('there is one key per field', () {
      expect(Settings.keys, hasLength(6));
      expect(Settings.keys.toSet(), hasLength(6));
    });
  });

  group('reading', () {
    test('an empty store yields the declared defaults', () {
      expect(Settings.read(MemoryStore()), const Settings());
    });

    test('stored values win over the defaults', () {
      final store = MemoryStore(<String, Object?>{
        Settings.themeKey: 'dark',
        Settings.launchCountKey: 12,
      });
      final settings = Settings.read(store);
      expect(settings.theme, 'dark');
      expect(settings.launchCount, 12);
      expect(settings.notifications, isTrue);
    });

    test('a value of the wrong type falls back instead of failing', () {
      // Two versions ago this was an int. The app still has to start.
      final store = MemoryStore(<String, Object?>{
        Settings.notificationsKey: 'yes',
        Settings.launchCountKey: 'many',
      });
      expect(Settings.read(store), const Settings());
    });

    test('an int is accepted where a double is expected', () {
      final store = MemoryStore(<String, Object?>{Settings.textScaleKey: 2});
      expect(Settings.read(store).textScale, 2.0);
    });

    test('an unparseable date falls back to null, not to a crash', () {
      final store = MemoryStore(<String, Object?>{
        Settings.lastSyncKey: 'the ides of March',
      });
      expect(Settings.read(store).lastSync, isNull);
    });

    test('a list drops the entries that are the wrong type', () {
      final store = MemoryStore(<String, Object?>{
        Settings.dismissedBannersKey: <Object?>['sale', 7, null, 'gdpr'],
      });
      expect(
        Settings.read(store).dismissedBanners,
        <String>['sale', 'gdpr'],
      );
    });
  });

  group('writing', () {
    test('round-trips through the store', () {
      final settings = Settings(
        notifications: false,
        theme: 'dark',
        launchCount: 3,
        textScale: 1.25,
        lastSync: DateTime.utc(2024, 6),
        dismissedBanners: const <String>['sale'],
      );
      final store = MemoryStore();
      settings.writeTo(store);
      expect(Settings.read(store), settings);
    });

    test('a DateTime is stored as text, which every store can hold', () {
      final store = MemoryStore();
      Settings(lastSync: DateTime.utc(2024, 6)).writeTo(store);
      expect(store.values[Settings.lastSyncKey], '2024-06-01T00:00:00.000Z');
    });

    test('clear wipes this namespace and nothing else', () {
      final store = MemoryStore(<String, Object?>{'other.key': 'keep me'});
      const Settings().writeTo(store);
      expect(store.values.length, greaterThan(1));

      Settings.clear(store);
      expect(store.values, <String, Object?>{'other.key': 'keep me'});
    });
  });

  test('settings are a value type, so a rebuild compares equal', () {
    expect(const Settings().copyWith(theme: 'dark'),
        const Settings(theme: 'dark'));
  });
}
