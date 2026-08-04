import 'package:flutter_test/flutter_test.dart';

import 'package:linkunbound/platform/windows/win_package_context.dart';
import 'package:linkunbound/platform/windows/win_registration_service.dart';

void main() {
  group('openWithExts', () {
    test('contains all expected file extensions', () {
      final exts = winRegistrationOpenWithExts;
      expect(
        exts,
        containsAll([
          '.htm',
          '.html',
          '.xhtml',
          '.xht',
          '.pdf',
          '.svg',
          '.mhtml',
          '.mht',
          '.shtml',
          '.webp',
        ]),
      );
    });

    test('has no duplicates', () {
      final exts = winRegistrationOpenWithExts;
      expect(exts.toSet().length, equals(exts.length));
    });

    test('every entry starts with a dot', () {
      for (final ext in winRegistrationOpenWithExts) {
        expect(ext, startsWith('.'), reason: '$ext should start with a dot');
      }
    });
  });
  group('progIdMatchesLinkUnbound', () {
    test('returns false for null', () {
      expect(progIdMatchesLinkUnbound(null), isFalse);
    });

    test('returns true for exact desktop ProgId', () {
      expect(progIdMatchesLinkUnbound('LinkUnboundURL'), isTrue);
    });

    test('returns true case-insensitively for owned ProgIds', () {
      expect(progIdMatchesLinkUnbound('linkunboundurl'), isTrue);
      expect(progIdMatchesLinkUnbound('LINKUNBOUNDURL'), isTrue);
    });

    test('returns true for EdgeProto ProgId', () {
      expect(progIdMatchesLinkUnbound('LinkUnboundEdgeProto'), isTrue);
      expect(progIdMatchesLinkUnbound('linkunboundedgeproto'), isTrue);
    });

    test('returns false for third-party ProgId that embeds our name', () {
      // Substring match must NOT trigger a false positive.
      expect(progIdMatchesLinkUnbound('somethingLinkunboundXYZ'), isFalse);
      expect(progIdMatchesLinkUnbound('LINKUNBOUND'), isFalse);
      expect(progIdMatchesLinkUnbound('linkunbound'), isFalse);
    });

    test('returns false for an unrelated ProgId', () {
      expect(progIdMatchesLinkUnbound('ChromeHTML'), isFalse);
    });

    test('returns false for opaque MSIX-style ProgId without name', () {
      // MSIX assigns ProgIds like "AppXabc123..." which do not embed identity.
      expect(progIdMatchesLinkUnbound('AppXabc123def456'), isFalse);
    });

    test('returns false for empty string', () {
      expect(progIdMatchesLinkUnbound(''), isFalse);
    });
  });

  group('userChoicePaths keys', () {
    // Verify that the registry path map covers the expected association keys
    // so that defaultAssociations can report them.
    test('includes http and https URL schemes', () {
      final keys = winRegistrationUserChoiceKeys;
      expect(keys, containsAll(['http', 'https']));
    });

    test('includes htm and html file extensions', () {
      final keys = winRegistrationUserChoiceKeys;
      expect(keys, containsAll(['.htm', '.html']));
    });

    test('includes xhtml and svg file extensions', () {
      final keys = winRegistrationUserChoiceKeys;
      expect(keys, containsAll(['.xhtml', '.svg']));
    });

    test('includes pdf file extension', () {
      expect(winRegistrationUserChoiceKeys, contains('.pdf'));
    });
  });

  group('isDevBuildPath', () {
    test('flags a Flutter build tree', () {
      // A build tree must never own the registration: it disappears on
      // `flutter clean` and the dead ProgId then shadows the real install.
      expect(
        isDevBuildPath(
          r'D:\Code\LinkUnbound\apps\linkunbound\build\windows'
          r'\x64\runner\Release\linkunbound.exe',
        ),
        isTrue,
      );
    });

    test('accepts forward slashes and mixed case', () {
      expect(
        isDevBuildPath('D:/Code/App/Build/Windows/x64/Runner/app.exe'),
        isTrue,
      );
    });

    test('does not flag a real installation', () {
      expect(
        isDevBuildPath(r'C:\Program Files\LinkUnbound\linkunbound.exe'),
        isFalse,
      );
      expect(
        isDevBuildPath(
          r'C:\Program Files\WindowsApps\rgdevment.LinkUnbound_1.0'
          r'\linkunbound.exe',
        ),
        isFalse,
      );
    });
  });
}
