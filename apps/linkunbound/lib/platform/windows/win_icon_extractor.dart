import 'dart:ffi';
import 'dart:io';
import 'dart:isolate';
import 'dart:typed_data';

import 'package:ffi/ffi.dart';
import 'package:image/image.dart' as img;
import 'package:linkunbound_core/linkunbound_core.dart';
import 'package:logging/logging.dart';
import 'package:win32/win32.dart';

final _log = Logger('WinIconExtractor');

const _iconSize = 32;

final class WinIconExtractor implements IconExtractor {
  @override
  Future<String> extractIcon(String executablePath, String outputPath) async {
    final outFile = File(outputPath);
    if (outFile.existsSync()) return outputPath;

    try {
      await outFile.parent.create(recursive: true);
    } on Object catch (e) {
      _log.warning('Could not create icon output dir: $e');
    }

    // Win32 GDI calls (PrivateExtractIcons, GetDIBits) must run on the calling
    // thread; pixel conversion + PNG encode are pure-Dart so offload them.
    final rawPixels = _extractRawPixels(executablePath);
    if (rawPixels == null) {
      throw IconExtractionException(executablePath, 'Win32 extraction failed');
    }

    final png = await Isolate.run(
      () => _encodeRgbaToPng(rawPixels.$1, rawPixels.$2, rawPixels.$3),
    );
    if (png == null) {
      throw IconExtractionException(executablePath, 'PNG encode failed');
    }

    try {
      outFile.writeAsBytesSync(png);
    } on Object catch (e) {
      throw IconExtractionException(executablePath, 'write failed: $e');
    }

    _log.fine('Extracted icon: $executablePath -> $outputPath');
    return outputPath;
  }
}

/// Returns (rgba bytes, width, height) extracted via Win32, or null on failure.
/// All GDI handles are released before this function returns.
(Uint8List, int, int)? _extractRawPixels(String executablePath) {
  final pathPtr = executablePath.toNativeUtf16();
  final hIconPtr = calloc<IntPtr>();
  final iconIdPtr = calloc<Uint32>();

  try {
    final extracted = PrivateExtractIcons(
      pathPtr,
      0,
      _iconSize,
      _iconSize,
      hIconPtr,
      iconIdPtr,
      1,
      0,
    );
    if (extracted == 0 || hIconPtr.value == 0) {
      _log.fine('PrivateExtractIcons returned $extracted for $executablePath');
      return null;
    }

    final hIcon = hIconPtr.value;
    try {
      return _iconToRgba(hIcon);
    } finally {
      DestroyIcon(hIcon);
    }
  } on Object catch (e, st) {
    _log.warning('Win32 icon extraction threw for $executablePath: $e', e, st);
    return null;
  } finally {
    calloc.free(pathPtr);
    calloc.free(hIconPtr);
    calloc.free(iconIdPtr);
  }
}

/// Extracts RGBA pixels from a GDI icon handle. All handles freed on return.
(Uint8List, int, int)? _iconToRgba(int hIcon) {
  final iconInfo = calloc<ICONINFO>();
  try {
    if (GetIconInfo(hIcon, iconInfo) == 0) {
      _log.fine('GetIconInfo failed');
      return null;
    }

    final hbmColor = iconInfo.ref.hbmColor;
    final hbmMask = iconInfo.ref.hbmMask;

    try {
      if (hbmColor == 0) {
        _log.fine('Monochrome icon (no color bitmap); skipping');
        return null;
      }

      final bmp = calloc<BITMAP>();
      try {
        if (GetObject(hbmColor, sizeOf<BITMAP>(), bmp.cast()) == 0) {
          _log.fine('GetObject on hbmColor failed');
          return null;
        }

        final width = bmp.ref.bmWidth;
        final height = bmp.ref.bmHeight;
        if (width <= 0 || height <= 0) return null;

        return _readBitmapPixels(hbmColor, width, height);
      } finally {
        calloc.free(bmp);
      }
    } finally {
      if (hbmColor != 0) DeleteObject(hbmColor);
      if (hbmMask != 0) DeleteObject(hbmMask);
    }
  } finally {
    calloc.free(iconInfo);
  }
}

/// Reads BGRA pixels via GDI, converts to RGBA, and returns the raw bytes
/// with dimensions. PNG encoding is left to the caller to run off-thread.
(Uint8List, int, int)? _readBitmapPixels(int hBitmap, int width, int height) {
  final hdc = GetDC(NULL);
  if (hdc == 0) return null;

  final bmi = calloc<BITMAPINFO>();
  final pixelBytes = width * height * 4;
  final pixels = calloc<Uint8>(pixelBytes);

  try {
    bmi.ref.bmiHeader.biSize = sizeOf<BITMAPINFOHEADER>();
    bmi.ref.bmiHeader.biWidth = width;
    bmi.ref.bmiHeader.biHeight = -height;
    bmi.ref.bmiHeader.biPlanes = 1;
    bmi.ref.bmiHeader.biBitCount = 32;
    bmi.ref.bmiHeader.biCompression = BI_RGB;
    bmi.ref.bmiHeader.biSizeImage = pixelBytes;

    final rows = GetDIBits(
      hdc,
      hBitmap,
      0,
      height,
      pixels.cast(),
      bmi,
      DIB_RGB_COLORS,
    );
    if (rows == 0) {
      _log.fine('GetDIBits returned 0 rows');
      return null;
    }

    final bgra = Uint8List.fromList(pixels.asTypedList(pixelBytes));

    // Convert BGRA → RGBA in-place.
    var anyAlpha = false;
    for (var i = 0; i < bgra.length; i += 4) {
      final b = bgra[i];
      final g = bgra[i + 1];
      final r = bgra[i + 2];
      final a = bgra[i + 3];
      bgra[i] = r;
      bgra[i + 1] = g;
      bgra[i + 2] = b;
      bgra[i + 3] = a;
      if (a != 0) anyAlpha = true;
    }
    if (!anyAlpha) {
      for (var i = 3; i < bgra.length; i += 4) {
        bgra[i] = 0xFF;
      }
    }

    return (bgra, width, height);
  } finally {
    calloc.free(pixels);
    calloc.free(bmi);
    ReleaseDC(NULL, hdc);
  }
}

/// Pure function: resize if needed and encode as PNG.
/// Runs in a separate isolate to keep the main thread free.
Uint8List? _encodeRgbaToPng(Uint8List rgba, int width, int height) {
  final image = img.Image.fromBytes(
    width: width,
    height: height,
    bytes: rgba.buffer,
    numChannels: 4,
    order: img.ChannelOrder.rgba,
  );

  final target = image.width > _iconSize || image.height > _iconSize
      ? img.copyResize(image, width: _iconSize, height: _iconSize)
      : image;

  return Uint8List.fromList(img.encodePng(target));
}

final class IconExtractionException implements Exception {
  const IconExtractionException(this.executablePath, this.reason);
  final String executablePath;
  final String reason;

  @override
  String toString() =>
      'IconExtractionException: Failed to extract icon from '
      '$executablePath: $reason';
}
