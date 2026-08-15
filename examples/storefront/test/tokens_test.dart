/// [catalogue.lerp]: interpolation that composes all the way down.
library;

import 'package:dmx/dmx.dart';
import 'package:dmx_storefront_example/tokens.dart';
import 'package:test/test.dart';

const light = ThemeTokens(
  palette: Rgba(red: 255, green: 255, blue: 255),
  motion: Motion(
    fast: Duration(milliseconds: 100),
    slow: Duration(milliseconds: 400),
    overshoot: 0.0,
  ),
  cornerRadius: 4.0,
  elevation: 0,
  fontFamily: 'Inter',
);

const dark = ThemeTokens(
  palette: Rgba(red: 0, green: 0, blue: 0, alpha: 0.8),
  motion: Motion(
    fast: Duration(milliseconds: 200),
    slow: Duration(milliseconds: 600),
    overshoot: 1.0,
  ),
  cornerRadius: 12.0,
  elevation: 8,
  fontFamily: 'Georgia',
);

void main() {
  test('t = 0 is the start and t = 1 is the end, exactly', () {
    expect(light.lerp(dark, 0), light);
    expect(light.lerp(dark, 1), dark);
  });

  test('numbers blend', () {
    final mid = light.lerp(dark, 0.5);
    expect(mid.cornerRadius, 8.0);
    expect(mid.elevation, 4);
  });

  test("nested @dmx('lerp') types blend by calling their own lerp", () {
    final mid = light.lerp(dark, 0.5);
    expect(
        mid.palette, const Rgba(red: 128, green: 128, blue: 128, alpha: 0.9));
    expect(mid.motion.fast, const Duration(milliseconds: 150));
  });

  test('durations blend at microsecond resolution', () {
    expect(
      light.motion.lerp(dark.motion, 0.25).slow,
      const Duration(milliseconds: 450),
    );
  });

  test('a type with no midpoint steps at the halfway mark', () {
    expect(light.lerp(dark, 0.49).fontFamily, 'Inter');
    expect(light.lerp(dark, 0.5).fontFamily, 'Georgia');
  });

  test('t is not clamped, because a spring curve overshoots', () {
    expect(light.lerp(dark, 1.5).cornerRadius, 16.0);
    expect(light.lerp(dark, -0.5).cornerRadius, 0.0);
  });

  test('int channels round rather than truncate', () {
    const a = Rgba(red: 0, green: 0, blue: 0);
    const b = Rgba(red: 1, green: 1, blue: 1);
    expect(a.lerp(b, 0.5).red, 1);
    expect(a.lerp(b, 0.4).red, 0);
  });

  test('the theme is still a model: it round-trips through JSON', () {
    expect(ThemeTokens.fromJson(dark.toJson()),
        Ok<ThemeTokens, DecodeError>(dark));
  });

  test('a whole-number JSON double still decodes', () {
    final json = dark.toJson()..['corner_radius'] = 12;
    expect(ThemeTokens.fromJson(json), Ok<ThemeTokens, DecodeError>(dark));
  });
}
