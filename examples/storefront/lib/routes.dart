// GENERATED REGIONS ARE MACHINE-OWNED. Edit above the divider, run `dmx build`.
//
// `@dmx('route')` / `@dmx('router')` [catalogue.route] — deep links that cannot be typo'd.
//
// A route is a string in most apps, which means every navigation is an
// unchecked concatenation and every deep link is a hand-written parse. Here the
// route is a class: `location` builds the URL from typed fields, `parse` takes
// a `Uri` and gives back a `Result`, and the two are generated from the same
// pattern so they can never disagree about what `/orders/:id/refund` means.
//
// The parser is a list pattern over `uri.pathSegments`. That is Dart doing the
// matching, not a regex over a URL — which is the same reason this repo parses
// Dart with a real grammar instead of a regex.

import 'package:dmx/dmx.dart';

/// Every screen the app can be at.
///
/// `@dmx('router')` reads the sibling `@dmx('route')` classes and writes one matcher over
/// all of them. Adding a route below is the whole edit.
@dmx('router')
sealed class AppRoute {
  const AppRoute();

  //#region
  /// Declared on the supertype so navigation code can take an `AppRoute` and
  /// still ask for its URL.
  String get location;

  /// Tries the routes in declaration order and takes the first whose shape
  /// matches. `Result<HomeRoute, …>` is a `Result<AppRoute, …>` — Dart's
  /// generics are covariant, so no re-wrapping is generated.
  static Result<AppRoute, RouteMismatch> match(Uri uri) =>
      switch (uri.pathSegments) {
        [] || [''] => HomeRoute.parse(uri),
        ['products'] => CatalogRoute.parse(uri),
        ['products', _] => ProductRoute.parse(uri),
        ['orders', _] => OrderRoute.parse(uri),
        ['orders', _, 'refund', _] => RefundRoute.parse(uri),
        _ => Err(RouteMismatch('AppRoute', uri.toString())),
      };

  /// Every pattern this router knows, in order — useful for a sitemap, a test
  /// that asserts coverage, or an error page that lists what does exist.
  static const List<String> patterns = <String>[
    HomeRoute.pattern,
    CatalogRoute.pattern,
    ProductRoute.pattern,
    OrderRoute.pattern,
    RefundRoute.pattern,
  ];
  //#endregion
}

/// `/`
@dmx('route', {'pattern': '/'})
@dmx('model', {'json': false, 'copyWith': false, 'toString': false})
final class HomeRoute extends AppRoute {
  const HomeRoute();

  //#region
  static const String pattern = '/';

  @override
  String get location =>
      '/';

  static Result<HomeRoute, RouteMismatch> parse(Uri uri) =>
      switch (uri.pathSegments) {
        [] || [''] =>
          Ok(HomeRoute(
          )),
        _ => Err(RouteMismatch(pattern, uri.toString())),
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is HomeRoute);

  @override
  int get hashCode => runtimeType.hashCode;
  //#endregion
}

/// `/products?tag=…&page=…`
///
/// Query parameters are typed and defaulted. An absent `page` is 1; a `page`
/// of `banana` is 1 as well, because a malformed query string is not worth a
/// crash on a cold start from a shared link.
@dmx('route', {'pattern': '/products'})
@dmx('model', {'json': false, 'copyWith': false, 'toString': false})
final class CatalogRoute extends AppRoute {
  const CatalogRoute({this.tag, this.page = 1});

  @dmx('query')
  final String? tag;

  @dmx('query')
  final int page;

  //#region
  static const String pattern = '/products';

  @override
  String get location =>
      dmxLocation(
        '/products',
        dmxQuery(<String, String?>{
          'tag': tag,
          'page': '$page',
        }),
      );

  static Result<CatalogRoute, RouteMismatch> parse(Uri uri) =>
      switch (uri.pathSegments) {
        ['products'] =>
          Ok(CatalogRoute(
            tag: uri.queryParameters['tag'],
            page: switch (uri.queryParameters['page']) { final String value => int.tryParse(value) ?? 1, null => 1 },
          )),
        _ => Err(RouteMismatch(pattern, uri.toString())),
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is CatalogRoute &&
          other.tag == tag &&
          other.page == page);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        tag,
        page,
      );
  //#endregion
}

/// `/products/:id?ref=…`
@dmx('route', {'pattern': '/products/:id'})
@dmx('model', {'json': false, 'copyWith': false, 'toString': false})
final class ProductRoute extends AppRoute {
  const ProductRoute({required this.id, this.ref});

  final String id;

  @dmx('query')
  final String? ref;

  //#region
  static const String pattern = '/products/:id';

  @override
  String get location =>
      dmxLocation(
        '/products/$id',
        dmxQuery(<String, String?>{
          'ref': ref,
        }),
      );

  static Result<ProductRoute, RouteMismatch> parse(Uri uri) =>
      switch (uri.pathSegments) {
        ['products', final String id] =>
          Ok(ProductRoute(
            id: id,
            ref: uri.queryParameters['ref'],
          )),
        _ => Err(RouteMismatch(pattern, uri.toString())),
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is ProductRoute &&
          other.id == id &&
          other.ref == ref);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        id,
        ref,
      );
  //#endregion
}

/// `/orders/:orderId`
@dmx('route', {'pattern': '/orders/:orderId'})
@dmx('model', {'json': false, 'copyWith': false, 'toString': false})
final class OrderRoute extends AppRoute {
  const OrderRoute({required this.orderId});

  final String orderId;

  //#region
  static const String pattern = '/orders/:orderId';

  @override
  String get location =>
      '/orders/$orderId';

  static Result<OrderRoute, RouteMismatch> parse(Uri uri) =>
      switch (uri.pathSegments) {
        ['orders', final String orderId] =>
          Ok(OrderRoute(
            orderId: orderId,
          )),
        _ => Err(RouteMismatch(pattern, uri.toString())),
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is OrderRoute &&
          other.orderId == orderId);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        orderId,
      );
  //#endregion
}

/// `/orders/:orderId/refund/:lineNumber`
///
/// A typed path segment: `lineNumber` is an `int` in Dart, so the parse fails
/// rather than handing a screen a number it cannot use.
@dmx('route', {'pattern': '/orders/:orderId/refund/:lineNumber'})
@dmx('model', {'json': false, 'copyWith': false, 'toString': false})
final class RefundRoute extends AppRoute {
  const RefundRoute({required this.orderId, required this.lineNumber});

  final String orderId;
  final int lineNumber;

  //#region
  static const String pattern = '/orders/:orderId/refund/:lineNumber';

  @override
  String get location =>
      '/orders/$orderId/refund/$lineNumber';

  static Result<RefundRoute, RouteMismatch> parse(Uri uri) =>
      switch (uri.pathSegments) {
        ['orders', final String orderId, 'refund', final String lineNumber] =>
          switch ((
            int.tryParse(lineNumber),
          )) {
            (
              final int lineNumber,
            ) =>
              Ok(RefundRoute(
                orderId: orderId,
                lineNumber: lineNumber,
              )),
            _ => Err(RouteMismatch(pattern, uri.toString())),
          },
        _ => Err(RouteMismatch(pattern, uri.toString())),
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is RefundRoute &&
          other.orderId == orderId &&
          other.lineNumber == lineNumber);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        orderId,
        lineNumber,
      );
  //#endregion
}
