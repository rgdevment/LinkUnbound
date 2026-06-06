import 'package:flutter_test/flutter_test.dart';

import 'package:linkunbound/platform/windows/win_browser_detector.dart';

void main() {
  group('extractExePath — quoted paths', () {
    test('extracts path from double-quoted command', () {
      expect(
        extractExePath(r'"C:\Program Files\Chrome\chrome.exe" "%1"'),
        r'C:\Program Files\Chrome\chrome.exe',
      );
    });

    test('extracts path when no closing quote', () {
      expect(extractExePath(r'"C:\chrome.exe'), r'C:\chrome.exe');
    });

    test('returns empty string for empty input', () {
      expect(extractExePath(''), '');
    });

    test('returns empty string for whitespace-only input', () {
      expect(extractExePath('   '), '');
    });
  });

  group('extractExePath — unquoted paths', () {
    test('extracts simple unquoted path with args', () {
      expect(extractExePath(r'C:\chrome.exe --flag'), r'C:\chrome.exe');
    });

    test('extracts simple unquoted path without args', () {
      expect(extractExePath(r'C:\chrome.exe'), r'C:\chrome.exe');
    });

    test('extracts path when args use %1 placeholder', () {
      expect(extractExePath(r'C:\chrome.exe %1'), r'C:\chrome.exe');
    });

    test('case-insensitive .EXE extension', () {
      expect(extractExePath(r'C:\CHROME.EXE --flag'), r'C:\CHROME.EXE');
    });

    test('path without .exe falls back to space split', () {
      expect(extractExePath(r'C:\launcher --go'), r'C:\launcher');
    });

    test('path without .exe and no space returns full string', () {
      expect(extractExePath(r'C:\launcher'), r'C:\launcher');
    });
  });

  group('extractExePath — intermediate .exe directory', () {
    test('does not truncate at intermediate .exe directory component', () {
      // The real executable is chrome.exe; the directory contains ".exe".
      expect(
        extractExePath(r'C:\some.exe.dir\chrome.exe --flag'),
        r'C:\some.exe.dir\chrome.exe',
      );
    });

    test('handles multiple .exe-containing directory levels', () {
      expect(
        extractExePath(r'C:\a.exe.d\b.exe.d\browser.exe %1'),
        r'C:\a.exe.d\b.exe.d\browser.exe',
      );
    });

    test(
      'quoted path with intermediate .exe directory is handled correctly',
      () {
        expect(
          extractExePath(r'"C:\some.exe.dir\chrome.exe" "%1"'),
          r'C:\some.exe.dir\chrome.exe',
        );
      },
    );
  });
}
