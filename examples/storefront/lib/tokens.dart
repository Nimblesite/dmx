// GENERATED REGIONS ARE MACHINE-OWNED. Edit above the divider, run `dmx build`.
//
// `@dmx('lerp')` [catalogue.lerp] — interpolation, for the whole tree, for free.
//
// Anyone who has written a Flutter `ThemeExtension` has written `lerp` by
// hand: twenty fields, each blended by type, and the one you forgot is the one
// that snaps mid-animation. It is pure boilerplate derived entirely from the
// field list, which makes it a generator's job.
//
// The interesting part is that interpolation *composes*. A field whose type
// also carries `@dmx('lerp')` is blended by calling its `lerp`, so a theme animates
// all the way down without a single line of hand-written traversal.

import 'package:dmx/dmx.dart';

/// A colour as four channels, so blending is channel-wise and correct — which
/// interpolating a packed `0xAARRGGBB` integer is not.
@dmx('model')
@dmx('lerp')
class Rgba {
  const Rgba({
    required this.red,
    required this.green,
    required this.blue,
    this.alpha = 1.0,
  });

  final int red;
  final int green;
  final int blue;
  final double alpha;

  //#region
  static Result<Rgba, DecodeError> fromJson(Object? json, [String path = 'Rgba']) =>
      switch (json) {
        {
          'red': final int red,
          'green': final int green,
          'blue': final int blue,
          'alpha': final num alpha,
        } =>
          Ok(Rgba(
            red: red,
            green: green,
            blue: blue,
            alpha: alpha.toDouble(),
          )),
        _ => Err(DecodeError(path, 'Rgba', json)),
      };

  Map<String, dynamic> toJson() => <String, dynamic>{
        'red': red,
        'green': green,
        'blue': blue,
        'alpha': alpha,
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Rgba &&
          other.red == red &&
          other.green == green &&
          other.blue == blue &&
          other.alpha == alpha);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        red,
        green,
        blue,
        alpha,
      );

  @override
  String toString() => 'Rgba(red: $red, green: $green, blue: $blue, alpha: $alpha)';

  Rgba copyWith({
    int? red,
    int? green,
    int? blue,
    double? alpha,
  }) =>
      Rgba(
        red: red ?? this.red,
        green: green ?? this.green,
        blue: blue ?? this.blue,
        alpha: alpha ?? this.alpha,
      );

  /// Blends towards [other] by `t`.
  ///
  /// `t` is not clamped: overshooting is what a spring curve does, and
  /// clamping it here would quietly flatten the animation somebody chose.
  /// Fields with no meaningful midpoint step at `t = 0.5` instead of
  /// pretending otherwise.
  Rgba lerp(Rgba other, double t) => Rgba(
        red: dmxLerpInt(red, other.red, t),
        green: dmxLerpInt(green, other.green, t),
        blue: dmxLerpInt(blue, other.blue, t),
        alpha: dmxLerpDouble(alpha, other.alpha, t),
      );
  //#endregion
}

/// How fast things move.
@dmx('model')
@dmx('lerp')
class Motion {
  const Motion({
    required this.fast,
    required this.slow,
    required this.overshoot,
  });

  final Duration fast;
  final Duration slow;
  final double overshoot;

  //#region
  static Result<Motion, DecodeError> fromJson(Object? json, [String path = 'Motion']) =>
      switch (json) {
        {
          'fast': final int fast,
          'slow': final int slow,
          'overshoot': final num overshoot,
        } =>
          Ok(Motion(
            fast: Duration(microseconds: fast),
            slow: Duration(microseconds: slow),
            overshoot: overshoot.toDouble(),
          )),
        _ => Err(DecodeError(path, 'Motion', json)),
      };

  Map<String, dynamic> toJson() => <String, dynamic>{
        'fast': fast.inMicroseconds,
        'slow': slow.inMicroseconds,
        'overshoot': overshoot,
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Motion &&
          other.fast == fast &&
          other.slow == slow &&
          other.overshoot == overshoot);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        fast,
        slow,
        overshoot,
      );

  @override
  String toString() => 'Motion(fast: $fast, slow: $slow, overshoot: $overshoot)';

  Motion copyWith({
    Duration? fast,
    Duration? slow,
    double? overshoot,
  }) =>
      Motion(
        fast: fast ?? this.fast,
        slow: slow ?? this.slow,
        overshoot: overshoot ?? this.overshoot,
      );

  /// Blends towards [other] by `t`.
  ///
  /// `t` is not clamped: overshooting is what a spring curve does, and
  /// clamping it here would quietly flatten the animation somebody chose.
  /// Fields with no meaningful midpoint step at `t = 0.5` instead of
  /// pretending otherwise.
  Motion lerp(Motion other, double t) => Motion(
        fast: dmxLerpDuration(fast, other.fast, t),
        slow: dmxLerpDuration(slow, other.slow, t),
        overshoot: dmxLerpDouble(overshoot, other.overshoot, t),
      );
  //#endregion
}

/// The whole theme.
///
/// [palette] and [motion] are themselves `@dmx('lerp')` types, so blending recurses;
/// [fontFamily] is a `String`, which has no midpoint, so it steps at `t = 0.5`
/// instead of pretending otherwise.
@dmx('model', {'fieldRename': 'snake'})
@dmx('lerp')
class ThemeTokens {
  const ThemeTokens({
    required this.palette,
    required this.motion,
    required this.cornerRadius,
    required this.elevation,
    required this.fontFamily,
  });

  final Rgba palette;
  final Motion motion;
  final double cornerRadius;
  final int elevation;
  final String fontFamily;

  //#region
  static Result<ThemeTokens, DecodeError> fromJson(Object? json, [String path = 'ThemeTokens']) =>
      switch (json) {
        {
          'palette': final Object? palette,
          'motion': final Object? motion,
          'corner_radius': final num cornerRadius,
          'elevation': final int elevation,
          'font_family': final String fontFamily,
        } =>
          switch ((
            Rgba.fromJson(palette, '$path.palette'),
            Motion.fromJson(motion, '$path.motion'),
          )) {
            (
              Ok(value: final palette),
              Ok(value: final motion),
            ) =>
              Ok(ThemeTokens(
                palette: palette,
                motion: motion,
                cornerRadius: cornerRadius.toDouble(),
                elevation: elevation,
                fontFamily: fontFamily,
              )),
            (Err(error: final e), _) => Err(e),
            (_, Err(error: final e)) => Err(e),
          },
        _ => Err(DecodeError(path, 'ThemeTokens', json)),
      };

  Map<String, dynamic> toJson() => <String, dynamic>{
        'palette': palette.toJson(),
        'motion': motion.toJson(),
        'corner_radius': cornerRadius,
        'elevation': elevation,
        'font_family': fontFamily,
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is ThemeTokens &&
          other.palette == palette &&
          other.motion == motion &&
          other.cornerRadius == cornerRadius &&
          other.elevation == elevation &&
          other.fontFamily == fontFamily);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        palette,
        motion,
        cornerRadius,
        elevation,
        fontFamily,
      );

  @override
  String toString() => 'ThemeTokens(palette: $palette, motion: $motion, cornerRadius: $cornerRadius, elevation: $elevation, fontFamily: $fontFamily)';

  ThemeTokens copyWith({
    Rgba? palette,
    Motion? motion,
    double? cornerRadius,
    int? elevation,
    String? fontFamily,
  }) =>
      ThemeTokens(
        palette: palette ?? this.palette,
        motion: motion ?? this.motion,
        cornerRadius: cornerRadius ?? this.cornerRadius,
        elevation: elevation ?? this.elevation,
        fontFamily: fontFamily ?? this.fontFamily,
      );

  /// Blends towards [other] by `t`.
  ///
  /// `t` is not clamped: overshooting is what a spring curve does, and
  /// clamping it here would quietly flatten the animation somebody chose.
  /// Fields with no meaningful midpoint step at `t = 0.5` instead of
  /// pretending otherwise.
  ThemeTokens lerp(ThemeTokens other, double t) => ThemeTokens(
        palette: palette.lerp(other.palette, t),
        motion: motion.lerp(other.motion, t),
        cornerRadius: dmxLerpDouble(cornerRadius, other.cornerRadius, t),
        elevation: dmxLerpInt(elevation, other.elevation, t),
        fontFamily: dmxLerpStep(fontFamily, other.fontFamily, t),
      );
  //#endregion
}
