import 'package:linkunbound_core/linkunbound_core.dart';
import 'package:test/test.dart';

void main() {
  group('stripEdgeProtocol', () {
    test('returns plain https URL unchanged', () {
      expect(stripEdgeProtocol('https://example.com'), 'https://example.com');
    });

    test('returns plain http URL unchanged', () {
      expect(stripEdgeProtocol('http://example.com'), 'http://example.com');
    });

    test('strips microsoft-edge: wrapping an https URL', () {
      expect(
        stripEdgeProtocol('microsoft-edge:https://example.com/path?q=1'),
        'https://example.com/path?q=1',
      );
    });

    test('strips microsoft-edge: wrapping an http URL', () {
      expect(
        stripEdgeProtocol('microsoft-edge:http://example.com'),
        'http://example.com',
      );
    });

    test('strips microsoft-edge-https:// prefix', () {
      expect(
        stripEdgeProtocol('microsoft-edge-https://example.com/page'),
        'https://example.com/page',
      );
    });

    test('is case-insensitive for the prefix', () {
      expect(
        stripEdgeProtocol('Microsoft-Edge:https://example.com'),
        'https://example.com',
      );
      expect(
        stripEdgeProtocol('MICROSOFT-EDGE-HTTPS://example.com'),
        'https://example.com',
      );
    });

    test('returns non-URL strings unchanged', () {
      expect(stripEdgeProtocol('--flag'), '--flag');
    });

    test('returns empty string unchanged', () {
      expect(stripEdgeProtocol(''), '');
    });

    test('preserves URL with query and fragment', () {
      const input = 'microsoft-edge:https://example.com/p?a=1&b=2#section';
      expect(stripEdgeProtocol(input), 'https://example.com/p?a=1&b=2#section');
    });
  });

  group('unwrapSafeLink', () {
    test('returns input unchanged when not a safe link', () {
      expect(
        unwrapSafeLink('https://example.com/page'),
        'https://example.com/page',
      );
    });

    test('returns input when host is unrelated', () {
      expect(
        unwrapSafeLink('https://other.example.com/?url=https://x.com'),
        'https://other.example.com/?url=https://x.com',
      );
    });

    test('unwraps Outlook SafeLink', () {
      const inner = 'https://example.com/report?id=7';
      final wrapped =
          'https://nam12.safelinks.protection.outlook.com/'
          '?url=${Uri.encodeComponent(inner)}';
      expect(unwrapSafeLink(wrapped), inner);
    });

    test('unwraps Teams CDN redirector', () {
      const inner = 'https://teams.example.com/document?id=1';
      final wrapped =
          'https://statics.teams.cdn.office.net/'
          '?url=${Uri.encodeComponent(inner)}';
      expect(unwrapSafeLink(wrapped), inner);
    });

    test('returns original when inner url parameter is missing', () {
      const wrapped = 'https://nam12.safelinks.protection.outlook.com/?other=1';
      expect(unwrapSafeLink(wrapped), wrapped);
    });

    test('returns original when inner url is empty', () {
      const wrapped = 'https://nam12.safelinks.protection.outlook.com/?url=';
      expect(unwrapSafeLink(wrapped), wrapped);
    });

    test('returns original when inner scheme is not http(s)', () {
      final wrapped =
          'https://nam12.safelinks.protection.outlook.com/'
          '?url=${Uri.encodeComponent('javascript:alert(1)')}';
      expect(unwrapSafeLink(wrapped), wrapped);
    });

    test('returns original when input is not parseable', () {
      const malformed = 'http://[::1';
      expect(unwrapSafeLink(malformed), malformed);
    });
  });

  group('isLaunchableUrl', () {
    test('accepts http and https', () {
      expect(isLaunchableUrl('http://example.com'), isTrue);
      expect(isLaunchableUrl('https://example.com/a?b=1'), isTrue);
    });

    test('accepts file URLs', () {
      expect(isLaunchableUrl('file:///C:/tmp/page.html'), isTrue);
    });

    test('rejects Chromium switches disguised as URLs', () {
      // These would be handed to the browser as argv and executed as switches.
      expect(isLaunchableUrl('--gpu-launcher=calc.exe'), isFalse);
      expect(isLaunchableUrl('--utility-cmd-prefix=calc.exe'), isFalse);
      expect(isLaunchableUrl('-foo'), isFalse);
      expect(isLaunchableUrl('/prefetch:1'), isFalse);
    });

    test('rejects dangerous schemes', () {
      expect(isLaunchableUrl('javascript:alert(1)'), isFalse);
      expect(isLaunchableUrl('data:text/html,<script>'), isFalse);
      expect(isLaunchableUrl('vbscript:msgbox'), isFalse);
    });

    test('rejects empty and unparseable input', () {
      expect(isLaunchableUrl(''), isFalse);
      expect(isLaunchableUrl('not a url'), isFalse);
    });

    test('is case-insensitive on the scheme', () {
      expect(isLaunchableUrl('HTTPS://example.com'), isTrue);
      expect(isLaunchableUrl('FILE:///C:/tmp/a.html'), isTrue);
    });
  });

  group('unwrapSafeLink double encoding', () {
    test('does not decode the inner URL a second time', () {
      // %2520 must stay a literal "%20" in the destination, not become a space.
      const inner = 'https://example.com/a%2520b';
      final wrapped =
          'https://nam12.safelinks.protection.outlook.com/'
          '?url=${Uri.encodeComponent(inner)}';
      expect(unwrapSafeLink(wrapped), inner);
    });
  });
}
