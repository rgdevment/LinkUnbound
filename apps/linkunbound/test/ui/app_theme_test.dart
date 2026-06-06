import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:linkunbound/ui/shared/app_theme.dart';

void main() {
  group('AppTheme.dark', () {
    test('returns a ThemeData with dark brightness', () {
      final theme = AppTheme.dark;
      expect(theme.brightness, Brightness.dark);
    });

    test('scaffoldBackgroundColor matches surface constant', () {
      final theme = AppTheme.dark;
      expect(theme.scaffoldBackgroundColor, const Color(0xFF1E1E2E));
    });

    test('primary color is the expected blue', () {
      final theme = AppTheme.dark;
      expect(theme.colorScheme.primary, const Color(0xFF89B4FA));
    });

    test('switchTheme thumbColor resolver returns primary when selected', () {
      final theme = AppTheme.dark;
      final thumbColor = theme.switchTheme.thumbColor;
      expect(thumbColor, isNotNull);
      final selected = thumbColor!.resolve({WidgetState.selected});
      expect(selected, const Color(0xFF89B4FA));
    });

    test(
      'switchTheme thumbColor resolver returns onSurfaceVariant when unselected',
      () {
        final theme = AppTheme.dark;
        final thumbColor = theme.switchTheme.thumbColor!;
        final unselected = thumbColor.resolve({});
        expect(unselected, const Color(0xFFA6ADC8));
      },
    );

    test(
      'switchTheme trackColor resolver returns primary with alpha when selected',
      () {
        final theme = AppTheme.dark;
        final trackColor = theme.switchTheme.trackColor;
        expect(trackColor, isNotNull);
        final selected = trackColor!.resolve({WidgetState.selected});
        // primary.withAlpha(80) = 0xFF89B4FA with alpha=80 → 0x5089B4FA
        expect(((selected?.a ?? 0) * 255.0).round(), 80);
      },
    );

    test('switchTheme trackColor resolver returns outline when unselected', () {
      final theme = AppTheme.dark;
      final trackColor = theme.switchTheme.trackColor!;
      final unselected = trackColor.resolve({});
      expect(unselected, const Color(0xFF45475A));
    });
  });

  group('AppTheme.light', () {
    test('returns a ThemeData with light brightness', () {
      final theme = AppTheme.light;
      expect(theme.brightness, Brightness.light);
    });

    test('scaffoldBackgroundColor matches light surface constant', () {
      final theme = AppTheme.light;
      expect(theme.scaffoldBackgroundColor, const Color(0xFFEFF1F5));
    });

    test('primary color is the expected blue', () {
      final theme = AppTheme.light;
      expect(theme.colorScheme.primary, const Color(0xFF1E66F5));
    });

    test(
      'switchTheme thumbColor resolver returns lightPrimary when selected',
      () {
        final theme = AppTheme.light;
        final thumbColor = theme.switchTheme.thumbColor;
        expect(thumbColor, isNotNull);
        final selected = thumbColor!.resolve({WidgetState.selected});
        expect(selected, const Color(0xFF1E66F5));
      },
    );

    test(
      'switchTheme trackColor resolver returns lightPrimary with alpha when selected',
      () {
        final theme = AppTheme.light;
        final trackColor = theme.switchTheme.trackColor!;
        final selected = trackColor.resolve({WidgetState.selected});
        expect(((selected?.a ?? 0) * 255.0).round(), 80);
      },
    );

    test(
      'switchTheme trackColor resolver returns lightOutline when unselected',
      () {
        final theme = AppTheme.light;
        final trackColor = theme.switchTheme.trackColor!;
        final unselected = trackColor.resolve({});
        expect(unselected, const Color(0xFFACACBC));
      },
    );
  });
}
