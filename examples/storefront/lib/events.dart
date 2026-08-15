// GENERATED REGIONS ARE MACHINE-OWNED. Edit above the divider, run `dmx build`.
//
// `@dmx('event')` [catalogue.events] — analytics that cannot be misspelled.
//
// Analytics code is stringly-typed by tradition: `log('prodct_viewed', {...})`
// ships, nobody notices for a quarter, and the funnel has a hole in it. Making
// each event a class moves the event name and its parameter names to exactly
// one place — a place the compiler can see.
//
// The generated members satisfy a hand-written interface, which is the whole
// trick: `AnalyticsEvent` below is ordinary Dart that a reviewer can read, and
// the region supplies the two members it demands.

import 'package:dmx/dmx.dart';

import 'payments.dart';

/// Anything that can be sent to an analytics backend.
///
/// Hand-written. `@dmx('event')` generates the implementations, not the contract.
sealed class AnalyticsEvent {
  const AnalyticsEvent();

  /// The event name as the backend knows it.
  String get name;

  /// A flat parameter map — flat because every analytics SDK in existence
  /// flattens it anyway, and doing it here means doing it deliberately.
  Map<String, Object?> get parameters;
}

/// Somebody looked at a product.
@dmx('event', {'name': 'product_viewed'})
final class ProductViewed extends AnalyticsEvent {
  const ProductViewed({
    required this.productId,
    required this.priceCents,
    required this.currency,
    this.referrer,
  });

  final String productId;
  final int priceCents;
  final String currency;
  final String? referrer;

  //#region
  static const String eventName = 'product_viewed';

  @override
  String get name => eventName;

  /// An absent optional parameter is *absent*, not null: a null in an
  /// analytics payload becomes a real bucket in most backends' dashboards.
  @override
  Map<String, Object?> get parameters => <String, Object?>{
        'product_id': productId,
        'price_cents': priceCents,
        'currency': currency,
        if (referrer case final String value) 'referrer': value,
      };
  //#endregion
}

/// Somebody added something to a cart.
@dmx('event', {'name': 'cart_item_added'})
final class CartItemAdded extends AnalyticsEvent {
  const CartItemAdded({
    required this.sku,
    required this.quantity,
    required this.cartValueCents,
  });

  final String sku;
  final int quantity;
  final int cartValueCents;

  //#region
  static const String eventName = 'cart_item_added';

  @override
  String get name => eventName;

  @override
  Map<String, Object?> get parameters => <String, Object?>{
        'sku': sku,
        'quantity': quantity,
        'cart_value_cents': cartValueCents,
      };
  //#endregion
}

/// Somebody paid.
///
/// The enum parameter encodes with the same wire name the API uses, because it
/// is the same enum — the analytics dashboard and the payments dashboard end
/// up agreeing without anyone maintaining a mapping table.
@dmx('event', {'name': 'checkout_completed'})
final class CheckoutCompleted extends AnalyticsEvent {
  const CheckoutCompleted({
    required this.orderId,
    required this.method,
    required this.totalCents,
    required this.lineCount,
    required this.usedDiscount,
  });

  final String orderId;
  final PaymentMethod method;
  final int totalCents;
  final int lineCount;
  final bool usedDiscount;

  //#region
  static const String eventName = 'checkout_completed';

  @override
  String get name => eventName;

  @override
  Map<String, Object?> get parameters => <String, Object?>{
        'order_id': orderId,
        'method': method.wire,
        'total_cents': totalCents,
        'line_count': lineCount,
        'used_discount': usedDiscount,
      };
  //#endregion
}

/// Something went wrong and we want to know how often.
@dmx('event', {'name': 'checkout_failed'})
final class CheckoutFailed extends AnalyticsEvent {
  const CheckoutFailed({
    required this.stage,
    required this.reason,
    this.httpStatus,
  });

  final String stage;
  final String reason;
  final int? httpStatus;

  //#region
  static const String eventName = 'checkout_failed';

  @override
  String get name => eventName;

  @override
  Map<String, Object?> get parameters => <String, Object?>{
        'stage': stage,
        'reason': reason,
        if (httpStatus case final int value) 'http_status': value,
      };
  //#endregion
}

/// Where events go. Hand-written, because sending them is your business.
abstract interface class AnalyticsSink {
  void record(String name, Map<String, Object?> parameters);
}

/// One call site for every event in the app.
extension AnalyticsDispatch on AnalyticsSink {
  void log(AnalyticsEvent event) => record(event.name, event.parameters);
}
