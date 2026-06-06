import 'package:flutter_test/flutter_test.dart';

import 'package:linkunbound/platform/cursor_locator.dart';

void main() {
  group('findDisplayForPoint', () {
    const single = [
      (originX: 0.0, originY: 0.0, width: 1920.0, height: 1080.0),
    ];

    test('returns the only display when cursor is inside it', () {
      final result = findDisplayForPoint(960, 540, single);
      expect(result, equals((0.0, 0.0, 1920.0, 1080.0)));
    });

    test('returns first display when list is empty (fallback)', () {
      final result = findDisplayForPoint(500, 500, []);
      expect(result, equals((0.0, 0.0, 1280.0, 800.0)));
    });

    test('returns primary display when cursor is outside all displays', () {
      // Cursor between monitors
      final displays = [
        (originX: 0.0, originY: 0.0, width: 1920.0, height: 1080.0),
        (originX: 1930.0, originY: 0.0, width: 1920.0, height: 1080.0),
      ];
      final result = findDisplayForPoint(1925, 540, displays);
      // Falls back to first display
      expect(result, equals((0.0, 0.0, 1920.0, 1080.0)));
    });

    test('selects secondary display when cursor is on it', () {
      final displays = [
        (originX: 0.0, originY: 0.0, width: 1920.0, height: 1080.0),
        (originX: 1920.0, originY: 0.0, width: 2560.0, height: 1440.0),
      ];
      final result = findDisplayForPoint(2000, 400, displays);
      expect(result, equals((1920.0, 0.0, 2560.0, 1440.0)));
    });

    test('handles negative origin (secondary display left of primary)', () {
      final displays = [
        (originX: 0.0, originY: 0.0, width: 1920.0, height: 1080.0),
        (originX: -2560.0, originY: 0.0, width: 2560.0, height: 1440.0),
      ];
      final result = findDisplayForPoint(-1000, 300, displays);
      expect(result, equals((-2560.0, 0.0, 2560.0, 1440.0)));
    });

    test('handles display above primary (negative Y origin)', () {
      final displays = [
        (originX: 0.0, originY: 0.0, width: 1920.0, height: 1080.0),
        (originX: 0.0, originY: -900.0, width: 1440.0, height: 900.0),
      ];
      final result = findDisplayForPoint(500, -200, displays);
      expect(result, equals((0.0, -900.0, 1440.0, 900.0)));
    });

    test('cursor exactly at display origin is considered inside', () {
      final result = findDisplayForPoint(0, 0, single);
      expect(result, equals((0.0, 0.0, 1920.0, 1080.0)));
    });

    test('cursor exactly at right/bottom edge is outside (half-open interval)',
        () {
      // The hit-test uses px < originX + width so exactly at 1920 falls back.
      final displays = [
        (originX: 0.0, originY: 0.0, width: 1920.0, height: 1080.0),
        (originX: 1920.0, originY: 0.0, width: 1920.0, height: 1080.0),
      ];
      final result = findDisplayForPoint(1920, 0, displays);
      // Falls on the boundary of the second display's left edge
      expect(result, equals((1920.0, 0.0, 1920.0, 1080.0)));
    });
  });
}
