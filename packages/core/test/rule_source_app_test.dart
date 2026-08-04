import 'dart:io';

import 'package:linkunbound_core/linkunbound_core.dart';
import 'package:test/test.dart';

void main() {
  late Directory tmp;
  late RuleService service;

  setUp(() async {
    tmp = await Directory.systemTemp.createTemp('lu_rule_source_');
    service = RuleService(rulesFile: File('${tmp.path}/rules.json'));
  });

  tearDown(() async {
    if (tmp.existsSync()) await tmp.delete(recursive: true);
  });

  group('lookupRule with an originating app', () {
    test('an app-scoped any-domain rule matches every link from that app', () {
      service.addRule(
        const Rule(domain: kAnyDomain, browserId: 'brave', sourceApp: 'slack'),
      );
      expect(
        service.lookupBrowser('https://anything.example', sourceApp: 'slack'),
        'brave',
      );
    });

    test('does not apply to links from other apps', () {
      service.addRule(
        const Rule(domain: kAnyDomain, browserId: 'brave', sourceApp: 'slack'),
      );
      expect(
        service.lookupBrowser('https://anything.example', sourceApp: 'teams'),
        isNull,
      );
      expect(service.lookupBrowser('https://anything.example'), isNull);
    });

    test('an app-scoped rule wins over a domain rule', () {
      // "Everything from Slack in Brave" is a deliberate statement about the
      // source; a generic domain rule must not quietly override it.
      service
        ..addRule(const Rule(domain: 'github.com', browserId: 'firefox'))
        ..addRule(
          const Rule(
            domain: kAnyDomain,
            browserId: 'brave',
            sourceApp: 'slack',
          ),
        );
      expect(
        service.lookupBrowser('https://github.com/x', sourceApp: 'slack'),
        'brave',
      );
      expect(service.lookupBrowser('https://github.com/x'), 'firefox');
    });

    test('a domain rule scoped to the app beats its any-domain rule', () {
      service
        ..addRule(
          const Rule(
            domain: kAnyDomain,
            browserId: 'brave',
            sourceApp: 'slack',
          ),
        )
        ..addRule(
          const Rule(
            domain: 'github.com',
            browserId: 'firefox',
            sourceApp: 'slack',
          ),
        );
      expect(
        service.lookupBrowser('https://github.com/x', sourceApp: 'slack'),
        'firefox',
      );
      expect(
        service.lookupBrowser('https://other.example', sourceApp: 'slack'),
        'brave',
      );
    });

    test('matching the origin is case-insensitive', () {
      service.addRule(
        const Rule(domain: kAnyDomain, browserId: 'brave', sourceApp: 'slack'),
      );
      expect(
        service.lookupBrowser('https://x.example', sourceApp: 'Slack'),
        'brave',
      );
    });

    test('subdomains still inherit within an app scope', () {
      service.addRule(
        const Rule(
          domain: 'example.com',
          browserId: 'firefox',
          sourceApp: 'slack',
        ),
      );
      expect(
        service.lookupBrowser('https://docs.example.com/a', sourceApp: 'slack'),
        'firefox',
      );
    });
  });

  group('rule identity', () {
    test('domain and origin together form the key', () {
      // Otherwise adding the app-scoped rule would silently replace the
      // domain one.
      service
        ..addRule(const Rule(domain: 'github.com', browserId: 'firefox'))
        ..addRule(
          const Rule(
            domain: 'github.com',
            browserId: 'brave',
            sourceApp: 'slack',
          ),
        );
      expect(service.rules, hasLength(2));
    });

    test('removing targets only the matching scope', () {
      service
        ..addRule(const Rule(domain: 'github.com', browserId: 'firefox'))
        ..addRule(
          const Rule(
            domain: 'github.com',
            browserId: 'brave',
            sourceApp: 'slack',
          ),
        )
        ..removeRule('github.com', sourceApp: 'slack');
      expect(service.rules, hasLength(1));
      expect(service.rules.single.sourceApp, isNull);
    });
  });

  group('persistence', () {
    test('origin and private flag survive a save/load cycle', () async {
      service.addRule(
        const Rule(
          domain: kAnyDomain,
          browserId: 'brave',
          sourceApp: 'slack',
          private: true,
        ),
      );
      await service.save();

      final reloaded = RuleService(rulesFile: File('${tmp.path}/rules.json'));
      await reloaded.load();
      final rule = reloaded.rules.single;
      expect(rule.sourceApp, 'slack');
      expect(rule.private, isTrue);
      expect(rule.matchesAnyDomain, isTrue);
    });

    test('rules written before this feature still load', () async {
      await File(
        '${tmp.path}/rules.json',
      ).writeAsString('[{"domain":"github.com","browserId":"firefox"}]');
      await service.load();
      expect(service.rules.single.sourceApp, isNull);
      expect(service.rules.single.private, isFalse);
      expect(service.lookupBrowser('https://github.com/x'), 'firefox');
    });
  });
}
