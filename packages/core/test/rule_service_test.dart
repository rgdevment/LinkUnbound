import 'dart:convert';
import 'dart:io';

import 'package:linkunbound_core/linkunbound_core.dart';
import 'package:test/test.dart';

void main() {
  late Directory tempDir;
  late File rulesFile;
  late RuleService service;

  setUp(() {
    tempDir = Directory.systemTemp.createTempSync('rule_service_test_');
    rulesFile = File('${tempDir.path}/rules.json');
    service = RuleService(rulesFile: rulesFile);
  });

  tearDown(() {
    if (tempDir.existsSync()) tempDir.deleteSync(recursive: true);
  });

  group('CRUD', () {
    test('addRule appends a new rule', () {
      service.addRule(const Rule(domain: 'github.com', browserId: 'chrome'));
      expect(service.rules, hasLength(1));
      expect(service.rules.first.domain, 'github.com');
    });

    test('addRule replaces existing rule for same domain', () {
      service.addRule(const Rule(domain: 'github.com', browserId: 'chrome'));
      service.addRule(const Rule(domain: 'github.com', browserId: 'firefox'));
      expect(service.rules, hasLength(1));
      expect(service.rules.first.browserId, 'firefox');
    });

    test('removeRule deletes by domain', () {
      service.addRule(const Rule(domain: 'a.com', browserId: 'x'));
      service.addRule(const Rule(domain: 'b.com', browserId: 'y'));
      service.removeRule('a.com');
      expect(service.rules, hasLength(1));
      expect(service.rules.first.domain, 'b.com');
    });

    test('removeRule is no-op for missing domain', () {
      service.addRule(const Rule(domain: 'a.com', browserId: 'x'));
      service.removeRule('nonexistent.com');
      expect(service.rules, hasLength(1));
    });

    test('updateRule changes browserId for existing domain', () {
      service.addRule(const Rule(domain: 'a.com', browserId: 'chrome'));
      service.updateRule('a.com', browserId: 'firefox');
      expect(service.rules.first.browserId, 'firefox');
    });

    test('updateRule leaves the other rules untouched', () {
      service
        ..addRule(const Rule(domain: 'a.com', browserId: 'chrome'))
        ..addRule(const Rule(domain: 'b.com', browserId: 'edge'))
        ..updateRule('a.com', browserId: 'firefox');
      expect(service.rules.map((r) => r.browserId), ['firefox', 'edge']);
    });

    test('updateRule toggles the private flag', () {
      service
        ..addRule(const Rule(domain: 'a.com', browserId: 'chrome'))
        ..updateRule('a.com', browserId: 'chrome', private: true);
      expect(service.rules.single.private, isTrue);
    });
  });

  group('lookupBrowser (hierarchical)', () {
    setUp(() {
      service.addRule(const Rule(domain: 'google.com', browserId: 'chrome'));
      service.addRule(
        const Rule(domain: 'mail.google.com', browserId: 'firefox'),
      );
      service.addRule(const Rule(domain: 'github.com', browserId: 'edge'));
    });

    test('exact subdomain match wins', () {
      expect(service.lookupBrowser('https://mail.google.com/inbox'), 'firefox');
    });

    test('falls back to parent domain', () {
      expect(
        service.lookupBrowser('https://docs.google.com/spreadsheets'),
        'chrome',
      );
    });

    test('deep subdomain walks up to parent', () {
      expect(
        service.lookupBrowser('https://sub.deep.google.com/page'),
        'chrome',
      );
    });

    test('returns null for unknown domain', () {
      expect(service.lookupBrowser('https://example.com'), isNull);
    });

    test('returns null for invalid url', () {
      expect(service.lookupBrowser('not a url'), isNull);
    });

    test('returns null for empty string', () {
      expect(service.lookupBrowser(''), isNull);
    });

    test('exact domain match with no subdomain', () {
      expect(service.lookupBrowser('https://github.com/org/repo'), 'edge');
    });
  });

  group('persistence (load/save)', () {
    test('save creates file and load restores rules', () async {
      service.addRule(const Rule(domain: 'a.com', browserId: 'chrome'));
      service.addRule(const Rule(domain: 'b.com', browserId: 'firefox'));
      await service.save();

      expect(rulesFile.existsSync(), isTrue);

      final fresh = RuleService(rulesFile: rulesFile);
      await fresh.load();
      expect(fresh.rules, hasLength(2));
      expect(fresh.rules.first.domain, 'a.com');
      expect(fresh.rules.last.browserId, 'firefox');
    });

    test('load with missing file yields empty list', () async {
      await service.load();
      expect(service.rules, isEmpty);
    });

    test('save produces valid JSON array', () async {
      service.addRule(const Rule(domain: 'x.com', browserId: 'y'));
      await service.save();
      final content = rulesFile.readAsStringSync();
      final decoded = jsonDecode(content) as List<dynamic>;
      expect(decoded, hasLength(1));
      expect((decoded.first as Map<String, dynamic>)['domain'], 'x.com');
    });

    test('load treats non-list root as empty', () async {
      rulesFile.writeAsStringSync('{"domain":"a.com","browserId":"chrome"}');
      await service.load();
      expect(service.rules, isEmpty);
    });

    test('load skips non-Map entries and keeps valid ones', () async {
      rulesFile.writeAsStringSync(
        jsonEncode([
          {'domain': 'good.com', 'browserId': 'chrome'},
          'not-a-map',
          {'domain': 'also-good.com', 'browserId': 'firefox'},
        ]),
      );
      await service.load();
      expect(service.rules, hasLength(2));
      expect(
        service.rules.map((r) => r.domain),
        containsAll(['good.com', 'also-good.com']),
      );
    });

    test('load skips entries missing required String fields', () async {
      rulesFile.writeAsStringSync(
        jsonEncode([
          {'domain': 'valid.com', 'browserId': 'edge'},
          {'domain': 123, 'browserId': 'chrome'},
          {'browserId': 'firefox'},
          {'domain': 'noBrowser.com'},
        ]),
      );
      await service.load();
      expect(service.rules, hasLength(1));
      expect(service.rules.first.domain, 'valid.com');
    });
  });
}
