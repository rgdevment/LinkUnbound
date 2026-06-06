import 'dart:ui' show Offset;

import 'package:screen_retriever/screen_retriever.dart';

abstract interface class CursorLocator {
  Future<(double, double)> cursorPosition();

  Future<(double, double)> screenSize();

  /// Visible rects of every display as (originX, originY, width, height).
  /// The caller hit-tests against an already-fetched cursor position so both
  /// queries can run concurrently without re-reading the cursor.
  Future<List<({double originX, double originY, double width, double height})>>
  displayRects();
}

final class ScreenRetrieverCursorLocator implements CursorLocator {
  const ScreenRetrieverCursorLocator();

  @override
  Future<(double, double)> cursorPosition() async {
    final Offset point = await screenRetriever.getCursorScreenPoint();
    return (point.dx, point.dy);
  }

  @override
  Future<(double, double)> screenSize() async {
    final display = await screenRetriever.getPrimaryDisplay();
    final size = display.visibleSize ?? display.size;
    return (size.width, size.height);
  }

  @override
  Future<List<({double originX, double originY, double width, double height})>>
  displayRects() async {
    final displays = await screenRetriever.getAllDisplays();
    return displays
        .map(
          (d) => (
            originX: d.visiblePosition?.dx ?? 0.0,
            originY: d.visiblePosition?.dy ?? 0.0,
            width: (d.visibleSize ?? d.size).width,
            height: (d.visibleSize ?? d.size).height,
          ),
        )
        .toList();
  }
}

/// Pure function for hit-testing a point against a list of display rects.
/// Extracted so it can be unit-tested without a window or platform channel.
///
/// Returns (originX, originY, width, height) of the first display whose
/// visible rect contains (px, py), or of the first display if none matches.
(double, double, double, double) findDisplayForPoint(
  double px,
  double py,
  List<({double originX, double originY, double width, double height})>
  displays,
) {
  if (displays.isEmpty) return (0, 0, 1280, 800);

  for (final d in displays) {
    if (px >= d.originX &&
        px < d.originX + d.width &&
        py >= d.originY &&
        py < d.originY + d.height) {
      return (d.originX, d.originY, d.width, d.height);
    }
  }
  // Cursor is outside all display rects (e.g. between monitors) — use first.
  final first = displays.first;
  return (first.originX, first.originY, first.width, first.height);
}
