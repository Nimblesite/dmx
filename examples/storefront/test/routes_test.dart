/// [catalogue.route]: build and parse the same URL from one pattern.
library;

import 'package:dmx/dmx.dart';
import 'package:dmx_storefront_example/routes.dart';
import 'package:test/test.dart';

void main() {
  group('building', () {
    test('a path segment is interpolated in place', () {
      expect(const ProductRoute(id: 'kettle').location, '/products/kettle');
    });

    test('an absent optional query parameter leaves no trace', () {
      expect(const CatalogRoute().location, '/products?page=1');
    });

    test('a present query parameter is encoded', () {
      expect(
        const CatalogRoute(tag: 'home & garden', page: 3).location,
        '/products?tag=home+%26+garden&page=3',
      );
    });

    test('a route with no parameters is a constant', () {
      expect(const HomeRoute().location, '/');
    });
  });

  group('parsing', () {
    test('round-trips every route through its own location', () {
      final routes = <AppRoute>[
        const HomeRoute(),
        const CatalogRoute(tag: 'home', page: 2),
        const ProductRoute(id: 'kettle', ref: 'newsletter'),
        const OrderRoute(orderId: 'o-1'),
        const RefundRoute(orderId: 'o-1', lineNumber: 2),
      ];
      for (final route in routes) {
        expect(
          AppRoute.match(Uri.parse(route.location)),
          Ok<AppRoute, RouteMismatch>(route),
          reason: '${route.location} did not round-trip',
        );
      }
    });

    test('a typed path segment must actually parse', () {
      expect(
        RefundRoute.parse(Uri.parse('/orders/o-1/refund/last')),
        isA<Err<RefundRoute, RouteMismatch>>(),
      );
      expect(
        RefundRoute.parse(Uri.parse('/orders/o-1/refund/2')),
        Ok<RefundRoute, RouteMismatch>(
          const RefundRoute(orderId: 'o-1', lineNumber: 2),
        ),
      );
    });

    test('a malformed query parameter falls back to its default', () {
      // A cold start from a shared link is the worst possible moment to crash.
      expect(
        CatalogRoute.parse(Uri.parse('/products?page=banana')),
        Ok<CatalogRoute, RouteMismatch>(const CatalogRoute()),
      );
    });

    test('an unknown path is a mismatch naming the router', () {
      expect(
        AppRoute.match(Uri.parse('/nope/at/all')),
        isA<Err<AppRoute, RouteMismatch>>()
            .having((e) => e.error.pattern, 'pattern', 'AppRoute'),
      );
    });

    test('a route refuses a URI that belongs to another route', () {
      expect(
        ProductRoute.parse(Uri.parse('/orders/o-1')),
        isA<Err<ProductRoute, RouteMismatch>>()
            .having((e) => e.error.pattern, 'pattern', '/products/:id'),
      );
    });

    test('the router prefers the more specific shape', () {
      expect(
        AppRoute.match(Uri.parse('/products')),
        isA<Ok<AppRoute, RouteMismatch>>()
            .having((r) => r.value, 'value', isA<CatalogRoute>()),
      );
      expect(
        AppRoute.match(Uri.parse('/products/kettle')),
        isA<Ok<AppRoute, RouteMismatch>>()
            .having((r) => r.value, 'value', isA<ProductRoute>()),
      );
    });
  });

  test('every route contributes its pattern to the router', () {
    expect(AppRoute.patterns, <String>[
      '/',
      '/products',
      '/products/:id',
      '/orders/:orderId',
      '/orders/:orderId/refund/:lineNumber',
    ]);
  });
}
