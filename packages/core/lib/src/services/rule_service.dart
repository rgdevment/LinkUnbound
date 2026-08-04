import 'dart:convert';
import 'dart:io';

import '../models/rule.dart';

final class RuleService {
  RuleService({required this.rulesFile});

  final File rulesFile;

  List<Rule> _rules = [];

  List<Rule> get rules => List.unmodifiable(_rules);

  Future<void> load() async {
    if (!rulesFile.existsSync()) {
      _rules = [];
      return;
    }
    final content = await rulesFile.readAsString();
    final raw = jsonDecode(content);
    if (raw is! List) {
      _rules = [];
      return;
    }
    // One corrupt entry must not discard the rest.
    _rules = raw.fold(<Rule>[], (acc, e) {
      if (e is! Map<String, dynamic>) return acc;
      if (e['domain'] is! String || e['browserId'] is! String) return acc;
      try {
        return [...acc, Rule.fromJson(e)];
      } catch (_) {
        return acc;
      }
    });
  }

  Future<void> save() async {
    await rulesFile.parent.create(recursive: true);
    const encoder = JsonEncoder.withIndent('  ');
    await rulesFile.writeAsString(
      encoder.convert(_rules.map((r) => r.toJson()).toList()),
    );
  }

  /// Rules are keyed by domain *and* originating app, so "github.com from
  /// Slack" and "github.com from anywhere" can coexist.
  void addRule(Rule rule) {
    _rules = [
      ..._rules.where(
        (r) => !(r.domain == rule.domain && r.sourceApp == rule.sourceApp),
      ),
      rule,
    ];
  }

  void removeRule(String domain, {String? sourceApp}) {
    _rules = _rules
        .where((r) => !(r.domain == domain && r.sourceApp == sourceApp))
        .toList();
  }

  void updateRule(
    String domain, {
    required String browserId,
    String? sourceApp,
    bool? private,
  }) {
    _rules = [
      for (final r in _rules)
        if (r.domain == domain && r.sourceApp == sourceApp)
          r.copyWith(browserId: browserId, private: private)
        else
          r,
    ];
  }

  String? lookupBrowser(String url, {String? sourceApp}) =>
      lookupRule(url, sourceApp: sourceApp)?.browserId;

  /// Finds the rule that governs [url] when opened from [sourceApp].
  ///
  /// A rule naming the originating app always wins over one that does not,
  /// even if the latter matches the domain more precisely: "everything from
  /// Slack in Brave" is a deliberate statement about the source, and a generic
  /// domain rule should not silently override it.
  Rule? lookupRule(String url, {String? sourceApp}) {
    final uri = Uri.tryParse(url);
    final host = uri?.host ?? '';
    final app = sourceApp?.toLowerCase();

    if (app != null) {
      final scoped = _rules.where((r) => r.sourceApp?.toLowerCase() == app);
      final byDomain = _matchHost(scoped, host);
      if (byDomain != null) return byDomain;
      final anyDomain = scoped.where((r) => r.matchesAnyDomain).firstOrNull;
      if (anyDomain != null) return anyDomain;
    }

    final unscoped = _rules.where((r) => r.sourceApp == null);
    final byDomain = _matchHost(unscoped, host);
    if (byDomain != null) return byDomain;
    return unscoped.where((r) => r.matchesAnyDomain).firstOrNull;
  }

  /// Walks up the domain hierarchy: a rule for `example.com` also covers
  /// `docs.example.com`.
  Rule? _matchHost(Iterable<Rule> candidates, String host) {
    if (host.isEmpty) return null;
    var current = host;
    while (true) {
      final exact = candidates.where((r) => r.domain == current).firstOrNull;
      if (exact != null) return exact;

      final dotIndex = current.indexOf('.');
      if (dotIndex < 0 || dotIndex == current.length - 1) return null;
      current = current.substring(dotIndex + 1);
    }
  }
}
