import 'dart:math';

import 'package:flutter/painting.dart';

/// Geometry of the picker window.
///
/// The window is sized before its first frame is laid out, so these constants
/// mirror what [PickerView] actually builds. They are split by what drives
/// each measurement — a fixed-size icon or a run of text — because the
/// text-driven parts grow with the system text-size setting and the fixed ones
/// do not. Folding them into one opaque overhead number is what let the footer
/// fall outside the window without anything failing loudly.
final class PickerLayout {
  const PickerLayout._();

  /// Content width, excluding the window frame.
  ///
  /// Wide enough for the app-scoped "always open" label beside the
  /// private-mode hint. At 300 that label and the URL both ellipsised on
  /// sight, which reads as a cropped window rather than a compact one.
  static const double width = 340.0;

  /// Rows beyond this scroll instead of making the window taller.
  static const int maxVisible = 6;

  // _BrowserRow: a 28px icon next to a single 13pt line.
  static const double _rowIcon = 28.0;
  static const double _rowText = 15.0;
  static const double _rowPadV = 8.0;

  // _UrlHeader: a 32px icon button next to two stacked lines (13pt + 11pt).
  static const double _headerButton = 32.0;
  static const double _headerText = 28.0;
  static const double _headerPadV = 22.0;

  // _AlwaysOpenFooter: an 18px checkbox next to a single 12pt line.
  static const double _footerBox = 18.0;
  static const double _footerText = 14.0;
  static const double _footerPadV = 16.0;

  static const double _dividers = 1.0;
  static const double _listPadV = 8.0;

  /// Frame left outside the client area by `TitleBarStyle.hidden`: the resize
  /// border survives even though the caption does not. Measured on Windows 11
  /// as GetWindowRect minus GetClientRect, not estimated.
  static const double _chromeW = 16.0;
  static const double _chromeH = 9.0;

  /// Past 2x the picker would be taller than most work areas, so the list
  /// scrolls instead of the window growing further.
  static const double _maxTextScale = 2.0;

  static double rowHeight([double textScale = 1.0]) =>
      max(_rowIcon, _rowText * _clamp(textScale)) + 2 * _rowPadV;

  /// Outer window size for [browserCount] browsers.
  ///
  /// [textScale] is the system text-size setting. Ignoring it clipped the
  /// footer as soon as Windows accessibility scaling went above 100%, which
  /// looks identical to the window simply being too small.
  static Size windowSize(int browserCount, {double textScale = 1.0}) {
    final scale = _clamp(textScale);
    final visible = min(browserCount, maxVisible);
    final header = max(_headerButton, _headerText * scale) + _headerPadV;
    final footer = max(_footerBox, _footerText * scale) + _footerPadV;
    final content =
        header + _dividers + _listPadV + visible * rowHeight(scale) + footer;
    return Size(width + _chromeW, content + _chromeH);
  }

  static double _clamp(double scale) =>
      scale.clamp(1.0, _maxTextScale).toDouble();
}
