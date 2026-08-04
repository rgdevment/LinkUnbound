import 'package:linkunbound_core/linkunbound_core.dart';
import 'package:test/test.dart';

void main() {
  group('privateModeArgs', () {
    test('uses --incognito for Chromium-family browsers', () {
      for (final path in [
        r'C:\Program Files\Google\Chrome\Application\chrome.exe',
        r'C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe',
        r'C:\Users\me\AppData\Local\Vivaldi\Application\vivaldi.exe',
        '/Applications/Chromium.app',
      ]) {
        expect(privateModeArgs(path), ['--incognito'], reason: path);
      }
    });

    test('uses -inprivate for Edge', () {
      // Edge is Chromium-based but spells the switch differently, so it must
      // be matched before the generic Chromium markers.
      expect(
        privateModeArgs(
          r'C:\Program Files\Microsoft\Edge\Application\msedge.exe',
        ),
        ['-inprivate'],
      );
    });

    test('uses -private-window for the Firefox family', () {
      expect(privateModeArgs(r'C:\Program Files\Mozilla Firefox\firefox.exe'), [
        '-private-window',
      ]);
      expect(privateModeArgs('/Applications/LibreWolf.app'), [
        '-private-window',
      ]);
    });

    test('returns nothing for browsers without a switch', () {
      // Safari has no command-line private mode; guessing one would open a
      // normal window with a stray argument.
      expect(privateModeArgs('/Applications/Safari.app'), isEmpty);
      expect(supportsPrivateMode('/Applications/Safari.app'), isFalse);
    });

    test('is case-insensitive', () {
      expect(privateModeArgs(r'C:\FIREFOX\FIREFOX.EXE'), ['-private-window']);
    });
  });

  group('Browser private mode', () {
    Browser browserAt(String path, {List<String>? privateArgs}) => Browser(
      id: 'b',
      name: 'B',
      executablePath: path,
      iconPath: '',
      privateArgs: privateArgs,
    );

    test('derives args from the executable when not overridden', () {
      final b = browserAt(r'C:\Program Files\Mozilla Firefox\firefox.exe');
      expect(b.resolvedPrivateArgs, ['-private-window']);
      expect(b.canOpenPrivately, isTrue);
    });

    test('an explicit empty list opts out', () {
      final b = browserAt(r'C:\chrome.exe', privateArgs: const []);
      expect(b.canOpenPrivately, isFalse);
    });

    test('an explicit list overrides the derived one', () {
      final b = browserAt(r'C:\chrome.exe', privateArgs: const ['--guest']);
      expect(b.resolvedPrivateArgs, ['--guest']);
    });

    test('survives a JSON round-trip', () {
      final b = browserAt(r'C:\chrome.exe', privateArgs: const ['--guest']);
      expect(Browser.fromJson(b.toJson()).resolvedPrivateArgs, ['--guest']);
      final derived = browserAt(r'C:\chrome.exe');
      expect(Browser.fromJson(derived.toJson()).privateArgs, isNull);
    });
  });
}
