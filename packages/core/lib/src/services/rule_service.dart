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

  void addRule(Rule rule) {
    _rules = [..._rules.where((r) => r.domain != rule.domain), rule];
  }

  void removeRule(String domain) {
    _rules = _rules.where((r) => r.domain != domain).toList();
  }

  void updateRule(String domain, {required String browserId}) {
    _rules = [
      for (final r in _rules)
        if (r.domain == domain) r.copyWith(browserId: browserId) else r,
    ];
  }

  String? lookupBrowser(String url) {
    final uri = Uri.tryParse(url);
    if (uri == null || uri.host.isEmpty) return null;
    return _lookupHierarchical(uri.host);
  }

  String? _lookupHierarchical(String host) {
    final exact = _rules.where((r) => r.domain == host).firstOrNull;
    if (exact != null) return exact.browserId;

    final dotIndex = host.indexOf('.');
    if (dotIndex < 0 || dotIndex == host.length - 1) return null;

    return _lookupHierarchical(host.substring(dotIndex + 1));
  }
}
