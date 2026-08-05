import 'package:flutter_test/flutter_test.dart';

import 'package:linkunbound/ui/picker/picker_layout.dart';

void main() {
  group('PickerLayout.windowSize', () {
    test('returns base overhead dimensions for 0 browsers', () {
      final size = PickerLayout.windowSize(0);
      expect(size.width, greaterThan(0));
      expect(size.height, greaterThan(0));
    });

    test('height grows by rowHeight for each browser up to maxVisible', () {
      final size1 = PickerLayout.windowSize(1);
      final size2 = PickerLayout.windowSize(2);
      expect(
        size2.height - size1.height,
        closeTo(PickerLayout.rowHeight(), 0.001),
      );
    });

    test('caps height at maxVisible rows', () {
      final sizeCapped = PickerLayout.windowSize(PickerLayout.maxVisible);
      final sizeOver = PickerLayout.windowSize(PickerLayout.maxVisible + 3);
      expect(sizeCapped.height, equals(sizeOver.height));
    });

    test('width is always the fixed picker width plus chrome', () {
      for (final count in [0, 1, 3, 6, 10]) {
        final size = PickerLayout.windowSize(count);
        expect(size.width, equals(PickerLayout.width + 16));
      }
    });

    test('leaves room for the header, footer and list padding', () {
      // Header 54 + dividers 1 + list padding 8 + footer 34 = 97, plus the
      // 9px of window chrome. Asserted as a whole so a change to any of the
      // pieces has to be a deliberate one.
      final size = PickerLayout.windowSize(1);
      expect(
        size.height,
        closeTo(97.0 + PickerLayout.rowHeight() + 9.0, 0.001),
      );
    });
  });

  group('PickerLayout text scaling', () {
    test('rowHeight is unchanged while the icon still dominates', () {
      // The 28px icon is taller than a 15px line until roughly 1.8x, so the
      // rows must not grow before that or the picker gains dead space.
      expect(PickerLayout.rowHeight(1.25), equals(PickerLayout.rowHeight()));
    });

    test('rowHeight grows once the text outgrows the icon', () {
      expect(
        PickerLayout.rowHeight(2.0),
        greaterThan(PickerLayout.rowHeight()),
      );
    });

    test('the window gets taller as the system text size grows', () {
      // Without this the footer is simply clipped: the window keeps its
      // 100%-scale height while every line inside it gets taller.
      final normal = PickerLayout.windowSize(3);
      final large = PickerLayout.windowSize(3, textScale: 1.5);
      expect(large.height, greaterThan(normal.height));
      expect(large.width, equals(normal.width));
    });

    test('scaling is clamped so the window cannot outgrow the screen', () {
      final atCap = PickerLayout.windowSize(6, textScale: 2.0);
      final beyond = PickerLayout.windowSize(6, textScale: 4.0);
      expect(beyond.height, equals(atCap.height));
    });

    test('a scale below 1 never shrinks the window', () {
      // Some platforms report 0 before the first metrics update.
      expect(
        PickerLayout.windowSize(3, textScale: 0).height,
        equals(PickerLayout.windowSize(3).height),
      );
    });
  });
}
