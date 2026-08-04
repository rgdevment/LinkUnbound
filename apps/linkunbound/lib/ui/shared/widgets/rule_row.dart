import 'package:flutter/material.dart';

import '../../../l10n/app_localizations.dart';

class RuleRow extends StatelessWidget {
  const RuleRow({
    required this.domain,
    required this.browserName,
    required this.browsers,
    required this.onBrowserChanged,
    required this.onDelete,
    this.sourceApp,
    this.private = false,
    super.key,
  });

  final String domain;
  final String browserName;
  final List<({String id, String name})> browsers;
  final void Function(String browserId) onBrowserChanged;
  final VoidCallback onDelete;

  /// Origin the rule is scoped to, when it targets an app rather than a domain.
  final String? sourceApp;
  final bool private;

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;
    final l10n = AppLocalizations.of(context)!;
    // An app-scoped rule reads as the app name; showing the literal "*" it is
    // stored under would be meaningless to the user.
    final label = sourceApp != null ? l10n.ruleFromApp(sourceApp!) : domain;

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          Expanded(
            flex: 3,
            child: Row(
              children: [
                if (sourceApp != null) ...[
                  Icon(
                    Icons.apps_outlined,
                    size: 14,
                    color: colors.onSurfaceVariant,
                  ),
                  const SizedBox(width: 6),
                ],
                Flexible(
                  child: Text(
                    label,
                    style: Theme.of(context).textTheme.bodyMedium,
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                if (private) ...[
                  const SizedBox(width: 6),
                  Icon(
                    Icons.visibility_off_outlined,
                    size: 14,
                    color: colors.onSurfaceVariant,
                  ),
                ],
              ],
            ),
          ),
          const SizedBox(width: 12),
          Expanded(
            flex: 3,
            child: DropdownButtonHideUnderline(
              child: DropdownButton<String>(
                value: browsers.any((b) => b.name == browserName)
                    ? browsers.firstWhere((b) => b.name == browserName).id
                    : null,
                isExpanded: true,
                isDense: true,
                dropdownColor: colors.surfaceBright,
                style: Theme.of(context).textTheme.bodyMedium,
                items: browsers
                    .map(
                      (b) => DropdownMenuItem(value: b.id, child: Text(b.name)),
                    )
                    .toList(),
                onChanged: (id) {
                  if (id != null) onBrowserChanged(id);
                },
              ),
            ),
          ),
          const SizedBox(width: 4),
          IconButton(
            onPressed: onDelete,
            icon: Icon(Icons.close, size: 16, color: colors.error),
            padding: EdgeInsets.zero,
            constraints: const BoxConstraints(minWidth: 32, minHeight: 32),
            splashRadius: 16,
            tooltip: AppLocalizations.of(context)!.deleteRuleTooltip,
          ),
        ],
      ),
    );
  }
}
