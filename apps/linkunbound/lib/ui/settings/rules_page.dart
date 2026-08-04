import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linkunbound_core/linkunbound_core.dart';

import '../../l10n/app_localizations.dart';
import '../../providers.dart';
import '../shared/widgets/base_dialog.dart';
import '../shared/widgets/group_card.dart';
import '../shared/widgets/rule_row.dart';
import '../shared/widgets/section_header.dart';

class RulesPage extends ConsumerWidget {
  const RulesPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final rules = ref.watch(rulesProvider);
    final browsers = ref.watch(browsersProvider);
    final l10n = AppLocalizations.of(context)!;

    final browserList = browsers.map((b) => (id: b.id, name: b.name)).toList();

    return ListView(
      padding: const EdgeInsets.fromLTRB(24, 20, 24, 24),
      children: [
        SectionHeader(label: l10n.sectionUrlRules),
        if (rules.isEmpty)
          GroupCard(
            child: Text(
              l10n.noRulesYet,
              style: Theme.of(context).textTheme.bodySmall,
            ),
          )
        else
          GroupCard(
            padding: const EdgeInsets.symmetric(vertical: 8, horizontal: 14),
            child: Column(
              children: [
                Padding(
                  padding: const EdgeInsets.only(bottom: 6),
                  child: Row(
                    children: [
                      Expanded(
                        flex: 3,
                        child: Text(
                          l10n.columnDomain,
                          style: Theme.of(context).textTheme.labelMedium,
                        ),
                      ),
                      const SizedBox(width: 12),
                      Expanded(
                        flex: 3,
                        child: Text(
                          l10n.columnBrowser,
                          style: Theme.of(context).textTheme.labelMedium,
                        ),
                      ),
                      const SizedBox(width: 36),
                    ],
                  ),
                ),
                ...rules.map(
                  (rule) => RuleRow(
                    domain: rule.domain,
                    sourceApp: rule.sourceApp,
                    private: rule.private,
                    browserName: _browserName(rule.browserId, browsers),
                    browsers: browserList,
                    onBrowserChanged: (browserId) {
                      ref
                          .read(rulesProvider.notifier)
                          .updateRule(
                            rule.domain,
                            browserId: browserId,
                            sourceApp: rule.sourceApp,
                          );
                    },
                    onDelete: () => _confirmDelete(
                      context,
                      ref,
                      rule.domain,
                      sourceApp: rule.sourceApp,
                    ),
                  ),
                ),
              ],
            ),
          ),
      ],
    );
  }

  String _browserName(String browserId, List<Browser> browsers) {
    for (final b in browsers) {
      if (b.id == browserId) return b.name;
    }
    return browserId;
  }

  void _confirmDelete(
    BuildContext context,
    WidgetRef ref,
    String domain, {
    String? sourceApp,
  }) {
    final l10n = AppLocalizations.of(context)!;
    showDialog<void>(
      context: context,
      builder: (ctx) => BaseDialog(
        title: l10n.deleteRuleTitle,
        content: l10n.deleteRuleContent(sourceApp ?? domain),
        confirmLabel: l10n.delete,
        confirmColor: Theme.of(ctx).colorScheme.error,
        onConfirm: () {
          ref
              .read(rulesProvider.notifier)
              .removeRule(domain, sourceApp: sourceApp);
          Navigator.of(ctx).pop();
        },
      ),
    );
  }
}
