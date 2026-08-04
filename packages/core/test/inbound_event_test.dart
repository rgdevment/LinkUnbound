import 'dart:convert';

import 'package:linkunbound_core/linkunbound_core.dart';
import 'package:test/test.dart';

void main() {
  group('OpenUrlEvent', () {
    test('encode produces correct JSON', () {
      const event = OpenUrlEvent('https://example.com');
      final json = jsonDecode(event.encode()) as Map<String, dynamic>;
      expect(json['action'], 'open_url');
      expect(json['url'], 'https://example.com');
    });

    test('decode round-trips', () {
      const original = OpenUrlEvent('https://test.com');
      final decoded = InboundEvent.decode(original.encode());
      expect(decoded, isA<OpenUrlEvent>());
      expect((decoded as OpenUrlEvent).url, 'https://test.com');
    });
  });

  group('ShowSettingsEvent', () {
    test('encode produces correct JSON', () {
      const event = ShowSettingsEvent();
      final json = jsonDecode(event.encode()) as Map<String, dynamic>;
      expect(json['action'], 'show_settings');
    });

    test('decode round-trips', () {
      const original = ShowSettingsEvent();
      final decoded = InboundEvent.decode(original.encode());
      expect(decoded, isA<ShowSettingsEvent>());
    });
  });

  group('decode errors', () {
    test('unknown action throws FormatException', () {
      expect(
        () => InboundEvent.decode('{"action": "unknown"}'),
        throwsA(isA<FormatException>()),
      );
    });

    test('missing action throws FormatException', () {
      expect(
        () => InboundEvent.decode('{"url": "https://x.com"}'),
        throwsA(isA<FormatException>()),
      );
    });

    // Anything arriving over IPC is untrusted. These used to raise TypeError,
    // which is not a FormatException, so it escaped the caller's catch and was
    // recorded as a crash instead of a discarded message.
    test('non-object JSON throws FormatException', () {
      expect(() => InboundEvent.decode('[]'), throwsA(isA<FormatException>()));
      expect(() => InboundEvent.decode('5'), throwsA(isA<FormatException>()));
      expect(
        () => InboundEvent.decode('"text"'),
        throwsA(isA<FormatException>()),
      );
    });

    test('open_url with a non-string url throws FormatException', () {
      expect(
        () => InboundEvent.decode('{"action":"open_url","url":1}'),
        throwsA(isA<FormatException>()),
      );
      expect(
        () => InboundEvent.decode('{"action":"open_url"}'),
        throwsA(isA<FormatException>()),
      );
    });

    test('malformed JSON throws FormatException', () {
      expect(
        () => InboundEvent.decode('{not json'),
        throwsA(isA<FormatException>()),
      );
    });
  });
}
