// GENERATED REGIONS ARE MACHINE-OWNED. Edit above the divider, run `dmx build`.
//
// `@dmx('prefs')` [catalogue.prefs] — typed settings over an untyped store.
//
// `SharedPreferences` and friends hand you `Object?` and a string key, so every
// app grows a settings wrapper: a key constant, a getter with a default, a
// setter, and a fallback for the day the stored value is the wrong type
// because two versions ago it was an `int`.
//
// The rule that makes this generator worth having is that **reading settings
// never fails**. A missing key, a value of the wrong type, a string that no
// longer parses — every one of them falls back to the declared default. An app
// that will not start because its preferences file is odd is a worse app than
// one that starts with the defaults.

import 'package:dmx/dmx.dart';

/// The untyped store. Hand-written, and deliberately tiny: implement it over
/// `SharedPreferences`, a file, or a `Map` in a test.
abstract interface class KeyValueStore {
  Object? read(String key);

  void write(String key, Object? value);

  void remove(String key);
}

/// Everything the app remembers between launches.
///
/// The constructor defaults are the source of truth for the fallbacks — one
/// place, and the same place a reader looks to find out what the default is.
@dmx('prefs', {'namespace': 'storefront'})
class Settings {
  const Settings({
    this.notifications = true,
    this.theme = 'system',
    this.launchCount = 0,
    this.textScale = 1.0,
    this.lastSync,
    this.dismissedBanners = const <String>[],
  });

  final bool notifications;
  final String theme;
  final int launchCount;
  final double textScale;
  final DateTime? lastSync;
  final List<String> dismissedBanners;

  //#region
  static const String namespace = 'storefront';

  static const String notificationsKey = 'storefront.notifications';
  static const String themeKey = 'storefront.theme';
  static const String launchCountKey = 'storefront.launch_count';
  static const String textScaleKey = 'storefront.text_scale';
  static const String lastSyncKey = 'storefront.last_sync';
  static const String dismissedBannersKey = 'storefront.dismissed_banners';

  static const List<String> keys = <String>[
    notificationsKey,
    themeKey,
    launchCountKey,
    textScaleKey,
    lastSyncKey,
    dismissedBannersKey,
  ];

  /// Total by construction: anything the store cannot supply as the right
  /// type falls back to the constructor default.
  static Settings read(KeyValueStore store) => Settings(
        notifications: switch (store.read(notificationsKey)) {
          final bool value => value,
          _ => true,
        },
        theme: switch (store.read(themeKey)) {
          final String value => value,
          _ => 'system',
        },
        launchCount: switch (store.read(launchCountKey)) {
          final int value => value,
          _ => 0,
        },
        textScale: switch (store.read(textScaleKey)) {
          final double value => value,
          final int value => value.toDouble(),
          _ => 1.0,
        },
        lastSync: switch (store.read(lastSyncKey)) {
          final String value => DateTime.tryParse(value),
          _ => null,
        },
        dismissedBanners: switch (store.read(dismissedBannersKey)) {
          final List<dynamic> value => <String>[
              for (final entry in value)
                if (entry case final String entry) entry,
            ],
          _ => const <String>[],
        },
      );

  /// The stored representation. `DateTime` goes down as ISO-8601 text because
  /// every one of these stores can hold a string, and not all of them can hold
  /// an `int` wide enough for microseconds.
  Map<String, Object?> get entries => <String, Object?>{
        notificationsKey: notifications,
        themeKey: theme,
        launchCountKey: launchCount,
        textScaleKey: textScale,
        lastSyncKey: lastSync?.toIso8601String(),
        dismissedBannersKey: dismissedBanners,
      };

  void writeTo(KeyValueStore store) => entries.forEach(store.write);

  /// Wipes only this namespace, so a "reset settings" button cannot take out
  /// somebody else's keys.
  static void clear(KeyValueStore store) => keys.forEach(store.remove);

  Settings copyWith({
    bool? notifications,
    String? theme,
    int? launchCount,
    double? textScale,
    DmxPatch<DateTime?> lastSync = const DmxKeep(),
    List<String>? dismissedBanners,
  }) =>
      Settings(
        notifications: notifications ?? this.notifications,
        theme: theme ?? this.theme,
        launchCount: launchCount ?? this.launchCount,
        textScale: textScale ?? this.textScale,
        lastSync: switch (lastSync) {
          DmxKeep() => this.lastSync,
          DmxTo(value: final value) => value
        },
        dismissedBanners: dismissedBanners ?? this.dismissedBanners,
      );

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Settings &&
          other.notifications == notifications &&
          other.theme == theme &&
          other.launchCount == launchCount &&
          other.textScale == textScale &&
          other.lastSync == lastSync &&
          dmxDeepEquals(other.dismissedBanners, dismissedBanners));

  @override
  int get hashCode => Object.hash(
        runtimeType,
        notifications,
        theme,
        launchCount,
        textScale,
        lastSync,
        dmxDeepHash(dismissedBanners),
      );

  @override
  String toString() =>
      'Settings(notifications: $notifications, theme: $theme, launchCount: $launchCount, textScale: $textScale, lastSync: $lastSync, dismissedBanners: $dismissedBanners)';
  //#endregion
}
