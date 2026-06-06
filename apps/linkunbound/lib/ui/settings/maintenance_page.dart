import 'dart:io' show Platform;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../l10n/app_localizations.dart';
import '../../platform/windows/win_package_context.dart';
import '../../providers.dart';
import '../shared/widgets/base_dialog.dart';
import '../shared/widgets/group_card.dart';
import '../shared/widgets/section_header.dart';

class MaintenancePage extends ConsumerWidget {
  const MaintenancePage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final colors = Theme.of(context).colorScheme;
    final l10n = AppLocalizations.of(context)!;
    final showUnregister = !isRunningInMsix();

    return ListView(
      padding: const EdgeInsets.fromLTRB(24, 20, 24, 24),
      children: [
        SectionHeader(label: l10n.sectionMaintenance),
        GroupCard(
          child: Column(
            children: [
              _ActionRow(
                icon: Icons.bug_report_outlined,
                label: l10n.exportDiagnosticsLabel,
                description: l10n.exportDiagnosticsDescription,
                color: colors.primary,
                onTap: () => _exportDiagnostics(context, ref),
              ),
              Divider(height: 1, color: colors.outline.withAlpha(40)),
              _ActionRow(
                icon: Icons.refresh,
                label: l10n.resetConfigLabel,
                description: l10n.resetConfigDescription,
                color: colors.error,
                onTap: () => _confirmReset(context, ref),
              ),
              if (showUnregister) ...[
                Divider(height: 1, color: colors.outline.withAlpha(40)),
                _ActionRow(
                  icon: Icons.delete_outline,
                  label: l10n.unregisterLabel,
                  description: l10n.unregisterDescription,
                  color: colors.error,
                  onTap: () => _confirmUnregister(context, ref),
                ),
              ],
            ],
          ),
        ),
      ],
    );
  }

  Future<void> _exportDiagnostics(BuildContext context, WidgetRef ref) async {
    final messenger = ScaffoldMessenger.maybeOf(context);
    final errorMsg = AppLocalizations.of(context)!.errorExportDiagnostics;
    showDialog<void>(
      context: context,
      barrierDismissible: false,
      builder: (_) => const Center(child: CircularProgressIndicator()),
    );

    var failed = false;
    try {
      final appDataDir = ref.read(appDataDirProvider);
      final version =
          ref.read(packageInfoProvider).valueOrNull?.version ?? 'unknown';

      final exporter = ref.read(diagnosticsExporterProvider);
      await exporter(appDataDir: appDataDir, appVersion: version);
    } on Object {
      failed = true;
    } finally {
      if (context.mounted) Navigator.of(context).pop();
      if (failed) {
        messenger?.showSnackBar(SnackBar(content: Text(errorMsg)));
      }
    }
  }

  void _confirmReset(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final messenger = ScaffoldMessenger.maybeOf(context);
    showDialog<void>(
      context: context,
      builder: (ctx) => BaseDialog(
        title: l10n.resetConfigTitle,
        content: l10n.resetConfigContent,
        confirmLabel: l10n.reset,
        confirmColor: Theme.of(ctx).colorScheme.error,
        onConfirm: () async {
          Navigator.of(ctx).pop();
          try {
            final browserService = ref.read(browserServiceProvider);
            await browserService.reset();
            await browserService.scanAndMerge();
            final iconsDir = ref.read(iconsDirProvider);
            final iconExtractor = ref.read(iconExtractorProvider);
            for (final browser in browserService.browsers) {
              try {
                await iconExtractor.extractIcon(
                  browser.executablePath,
                  '${iconsDir.path}${Platform.pathSeparator}${browser.id}.png',
                );
              } on Exception {
                // Best-effort
              }
            }
            ref.invalidate(browsersProvider);
            ref.invalidate(rulesProvider);
          } on Object {
            messenger?.showSnackBar(
              SnackBar(content: Text(l10n.errorResetConfig)),
            );
          }
        },
      ),
    );
  }

  void _confirmUnregister(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final messenger = ScaffoldMessenger.maybeOf(context);
    showDialog<void>(
      context: context,
      builder: (ctx) => BaseDialog(
        title: l10n.unregisterTitle,
        content: l10n.unregisterContent,
        confirmLabel: l10n.unregisterAction,
        confirmColor: Theme.of(ctx).colorScheme.error,
        onConfirm: () async {
          Navigator.of(ctx).pop();
          try {
            await ref.read(registrationServiceProvider).unregister();
          } on Object {
            messenger?.showSnackBar(
              SnackBar(content: Text(l10n.errorUnregister)),
            );
          } finally {
            ref.invalidate(isDefaultBrowserProvider);
          }
        },
      ),
    );
  }
}

class _ActionRow extends StatelessWidget {
  const _ActionRow({
    required this.icon,
    required this.label,
    required this.description,
    required this.color,
    required this.onTap,
  });

  final IconData icon;
  final String label;
  final String description;
  final Color color;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(6),
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 10),
        child: Row(
          children: [
            Icon(icon, size: 18, color: color),
            const SizedBox(width: 12),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    label,
                    style: Theme.of(
                      context,
                    ).textTheme.bodyMedium?.copyWith(color: color),
                  ),
                  Text(
                    description,
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                ],
              ),
            ),
            Icon(Icons.chevron_right, size: 18, color: color.withAlpha(120)),
          ],
        ),
      ),
    );
  }
}
