// GENERATED REGIONS ARE MACHINE-OWNED. Edit above the divider, run `dmx build`.
//
// `@dmx('union')` [catalogue.union] — a tagged sum type that decodes.
//
// This is the macro that proves the front end is relational. `@dmx('union')` on the
// sealed base reads its *sibling declarations* to find the variants — no type
// resolver, no analyzer, no build graph: the file is the scope, and a name is
// a name [frontend.name-index]. From that it writes the discriminated decode,
// the exhaustive `when`, and the narrowing accessors, and every variant keeps
// its own `@dmx('model')` codec.
//
// Dart's `sealed` already gives you exhaustive `switch`. What it does not give
// you is a decoder that turns `{"type": "placed", ...}` into the right variant,
// and that is the part everybody hand-writes and gets wrong.

import 'package:dmx/dmx.dart';

import 'catalog.dart';
import 'payments.dart';

/// One line of an order.
@dmx('model', {'fieldRename': 'snake'})
class OrderLine {
  const OrderLine({
    required this.sku,
    required this.quantity,
    required this.unitPrice,
  });

  final String sku;
  final int quantity;
  final Money unitPrice;

  //#region
  static Result<OrderLine, DecodeError> fromJson(Object? json, [String path = 'OrderLine']) =>
      switch (json) {
        {
          'sku': final String sku,
          'quantity': final int quantity,
          'unit_price': final Object? unitPrice,
        } =>
          switch ((
            Money.fromJson(unitPrice, '$path.unit_price'),
          )) {
            (
              Ok(value: final unitPrice),
            ) =>
              Ok(OrderLine(
                sku: sku,
                quantity: quantity,
                unitPrice: unitPrice,
              )),
            (Err(error: final e),) => Err(e),
          },
        _ => Err(DecodeError(path, 'OrderLine', json)),
      };

  Map<String, dynamic> toJson() => <String, dynamic>{
        'sku': sku,
        'quantity': quantity,
        'unit_price': unitPrice.toJson(),
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is OrderLine &&
          other.sku == sku &&
          other.quantity == quantity &&
          other.unitPrice == unitPrice);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        sku,
        quantity,
        unitPrice,
      );

  @override
  String toString() => 'OrderLine(sku: $sku, quantity: $quantity, unitPrice: $unitPrice)';

  OrderLine copyWith({
    String? sku,
    int? quantity,
    Money? unitPrice,
  }) =>
      OrderLine(
        sku: sku ?? this.sku,
        quantity: quantity ?? this.quantity,
        unitPrice: unitPrice ?? this.unitPrice,
      );
  //#endregion
}

/// Where an order is in its life.
///
/// The variants are the sibling classes below that extend this one. `@dmx('union')`
/// finds them by name, in this file, and writes the codec across all of them.
@dmx('union', {'discriminator': 'type', 'fieldRename': 'snake'})
sealed class OrderState {
  const OrderState();

  //#region
  /// Declared on the supertype so `value.toJson()` type-checks without a
  /// narrowing switch at every call site. Each variant generates the
  /// implementation, discriminator included.
  Map<String, dynamic> toJson();

  /// Dispatch on the discriminator, then hand the whole object — tag and all —
  /// to the variant's own decoder.
  static Result<OrderState, DecodeError> fromJson(
    Object? json, [
    String path = 'OrderState',
  ]) =>
      switch (json) {
        { 'type': 'draft' } => Draft.fromJson(json, '$path(draft)'),
        { 'type': 'placed' } => Placed.fromJson(json, '$path(placed)'),
        { 'type': 'shipped' } => Shipped.fromJson(json, '$path(shipped)'),
        { 'type': 'cancelled' } => Cancelled.fromJson(json, '$path(cancelled)'),
        { 'type': final Object? type } =>
          Err(DecodeError('$path.type', 'OrderState', type)),
        _ => Err(DecodeError(path, 'OrderState', json)),
      };

  /// Exhaustive by construction: adding a variant to this file makes every
  /// existing call to [when] a compile error until it is handled.
  T when<T>({
    required T Function(Draft value) draft,
    required T Function(Placed value) placed,
    required T Function(Shipped value) shipped,
    required T Function(Cancelled value) cancelled,
  }) =>
      switch (this) {
        final Draft value => draft(value),
        final Placed value => placed(value),
        final Shipped value => shipped(value),
        final Cancelled value => cancelled(value),
      };

  /// The same dispatch with every arm optional. Written with a null-check
  /// pattern rather than `?.call() ?? orElse()`, which would silently take the
  /// fallback whenever a handler legitimately returned null.
  T maybeWhen<T>({
    T Function(Draft value)? draft,
    T Function(Placed value)? placed,
    T Function(Shipped value)? shipped,
    T Function(Cancelled value)? cancelled,
    required T Function() orElse,
  }) =>
      switch (this) {
        final Draft value => switch (draft) {
            final handler? => handler(value),
            null => orElse(),
          },
        final Placed value => switch (placed) {
            final handler? => handler(value),
            null => orElse(),
          },
        final Shipped value => switch (shipped) {
            final handler? => handler(value),
            null => orElse(),
          },
        final Cancelled value => switch (cancelled) {
            final handler? => handler(value),
            null => orElse(),
          },
      };

  bool get isDraft => this is Draft;
  bool get isPlaced => this is Placed;
  bool get isShipped => this is Shipped;
  bool get isCancelled => this is Cancelled;

  /// Narrowing accessors, so a caller that already knows the variant does not
  /// need a `switch` to say so — and never needs a cast to prove it.
  Draft? get asDraft => switch (this) {
        final Draft value => value,
        _ => null,
      };
  Placed? get asPlaced => switch (this) {
        final Placed value => value,
        _ => null,
      };
  Shipped? get asShipped => switch (this) {
        final Shipped value => value,
        _ => null,
      };
  Cancelled? get asCancelled => switch (this) {
        final Cancelled value => value,
        _ => null,
      };
  //#endregion
}

/// Nothing is committed yet; the cart is still editable.
@dmx('model', {'fieldRename': 'snake'})
final class Draft extends OrderState {
  const Draft({required this.cartId, required this.lines});

  final String cartId;
  final List<OrderLine> lines;

  //#region
  static Result<Draft, DecodeError> fromJson(Object? json, [String path = 'Draft']) =>
      switch (json) {
        {
          'cart_id': final String cartId,
          'lines': final List<dynamic> lines,
        } =>
          switch ((
            dmxList<OrderLine>(lines, '$path.lines', OrderLine.fromJson),
          )) {
            (
              Ok(value: final lines),
            ) =>
              Ok(Draft(
                cartId: cartId,
                lines: lines,
              )),
            (Err(error: final e),) => Err(e),
          },
        _ => Err(DecodeError(path, 'Draft', json)),
      };

  @override
  Map<String, dynamic> toJson() => <String, dynamic>{
        'type': 'draft',
        'cart_id': cartId,
        'lines': lines.map((e0) => e0.toJson()).toList(),
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Draft &&
          other.cartId == cartId &&
          dmxDeepEquals(other.lines, lines));

  @override
  int get hashCode => Object.hash(
        runtimeType,
        cartId,
        dmxDeepHash(lines),
      );

  @override
  String toString() => 'Draft(cartId: $cartId, lines: $lines)';

  Draft copyWith({
    String? cartId,
    List<OrderLine>? lines,
  }) =>
      Draft(
        cartId: cartId ?? this.cartId,
        lines: lines ?? this.lines,
      );
  //#endregion
}

/// Paid for, not yet dispatched.
@dmx('model', {'fieldRename': 'snake'})
final class Placed extends OrderState {
  const Placed({
    required this.orderId,
    required this.placedAt,
    required this.method,
    required this.total,
  });

  final String orderId;
  final DateTime placedAt;
  final PaymentMethod method;
  final Money total;

  //#region
  static Result<Placed, DecodeError> fromJson(Object? json, [String path = 'Placed']) =>
      switch (json) {
        {
          'order_id': final String orderId,
          'placed_at': final String placedAt,
          'method': final Object? method,
          'total': final Object? total,
        } =>
          switch ((
            switch (DateTime.tryParse(placedAt)) { final DateTime parsed => Ok<DateTime, DecodeError>(parsed), null => Err<DateTime, DecodeError>(DecodeError('$path.placed_at', 'DateTime', placedAt)) },
            PaymentMethod.fromJson(method, '$path.method'),
            Money.fromJson(total, '$path.total'),
          )) {
            (
              Ok(value: final placedAt),
              Ok(value: final method),
              Ok(value: final total),
            ) =>
              Ok(Placed(
                orderId: orderId,
                placedAt: placedAt,
                method: method,
                total: total,
              )),
            (Err(error: final e), _, _) => Err(e),
            (_, Err(error: final e), _) => Err(e),
            (_, _, Err(error: final e)) => Err(e),
          },
        _ => Err(DecodeError(path, 'Placed', json)),
      };

  @override
  Map<String, dynamic> toJson() => <String, dynamic>{
        'type': 'placed',
        'order_id': orderId,
        'placed_at': placedAt.toIso8601String(),
        'method': method.toJson(),
        'total': total.toJson(),
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Placed &&
          other.orderId == orderId &&
          other.placedAt == placedAt &&
          other.method == method &&
          other.total == total);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        orderId,
        placedAt,
        method,
        total,
      );

  @override
  String toString() => 'Placed(orderId: $orderId, placedAt: $placedAt, method: $method, total: $total)';

  Placed copyWith({
    String? orderId,
    DateTime? placedAt,
    PaymentMethod? method,
    Money? total,
  }) =>
      Placed(
        orderId: orderId ?? this.orderId,
        placedAt: placedAt ?? this.placedAt,
        method: method ?? this.method,
        total: total ?? this.total,
      );
  //#endregion
}

/// In the hands of a carrier.
@dmx('model', {'fieldRename': 'snake'})
final class Shipped extends OrderState {
  const Shipped({
    required this.orderId,
    required this.carrier,
    required this.trackingNumber,
  });

  final String orderId;
  final String carrier;
  final String trackingNumber;

  //#region
  static Result<Shipped, DecodeError> fromJson(Object? json, [String path = 'Shipped']) =>
      switch (json) {
        {
          'order_id': final String orderId,
          'carrier': final String carrier,
          'tracking_number': final String trackingNumber,
        } =>
          Ok(Shipped(
            orderId: orderId,
            carrier: carrier,
            trackingNumber: trackingNumber,
          )),
        _ => Err(DecodeError(path, 'Shipped', json)),
      };

  @override
  Map<String, dynamic> toJson() => <String, dynamic>{
        'type': 'shipped',
        'order_id': orderId,
        'carrier': carrier,
        'tracking_number': trackingNumber,
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Shipped &&
          other.orderId == orderId &&
          other.carrier == carrier &&
          other.trackingNumber == trackingNumber);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        orderId,
        carrier,
        trackingNumber,
      );

  @override
  String toString() => 'Shipped(orderId: $orderId, carrier: $carrier, trackingNumber: $trackingNumber)';

  Shipped copyWith({
    String? orderId,
    String? carrier,
    String? trackingNumber,
  }) =>
      Shipped(
        orderId: orderId ?? this.orderId,
        carrier: carrier ?? this.carrier,
        trackingNumber: trackingNumber ?? this.trackingNumber,
      );
  //#endregion
}

/// Called off, with the reason kept for the finance team.
@dmx('model', {'fieldRename': 'snake'})
final class Cancelled extends OrderState {
  const Cancelled({
    required this.orderId,
    required this.reason,
    this.note,
  });

  final String orderId;
  final RefundReason reason;
  final String? note;

  //#region
  static Result<Cancelled, DecodeError> fromJson(Object? json, [String path = 'Cancelled']) =>
      switch (json) {
        {
          'order_id': final String orderId,
          'reason': final Object? reason,
        } =>
          switch ((
            RefundReason.fromJson(reason, '$path.reason'),
            dmxNullable<String>(dmxKey(json, 'note'), '$path.note', (value, path) => switch (value) {
              final String value => Ok(value),
              _ => Err(DecodeError(path, 'String', value)),
            }),
          )) {
            (
              Ok(value: final reason),
              Ok(value: final note),
            ) =>
              Ok(Cancelled(
                orderId: orderId,
                reason: reason,
                note: note,
              )),
            (Err(error: final e), _) => Err(e),
            (_, Err(error: final e)) => Err(e),
          },
        _ => Err(DecodeError(path, 'Cancelled', json)),
      };

  @override
  Map<String, dynamic> toJson() => <String, dynamic>{
        'type': 'cancelled',
        'order_id': orderId,
        'reason': reason.toJson(),
        'note': note,
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Cancelled &&
          other.orderId == orderId &&
          other.reason == reason &&
          other.note == note);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        orderId,
        reason,
        note,
      );

  @override
  String toString() => 'Cancelled(orderId: $orderId, reason: $reason, note: $note)';

  Cancelled copyWith({
    String? orderId,
    RefundReason? reason,
    DmxPatch<String?> note = const DmxKeep(),
  }) =>
      Cancelled(
        orderId: orderId ?? this.orderId,
        reason: reason ?? this.reason,
        note: switch (note) { DmxKeep() => this.note, DmxTo(value: final value) => value },
      );
  //#endregion
}
