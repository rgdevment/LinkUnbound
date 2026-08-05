import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:url_launcher/url_launcher.dart';

import '../../l10n/app_localizations.dart';
import '../../providers.dart';
import '../shared/widgets/group_card.dart';
import '../shared/widgets/section_header.dart';

class AboutPage extends ConsumerWidget {
  const AboutPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final colors = Theme.of(context).colorScheme;
    final l10n = AppLocalizations.of(context)!;

    return ListView(
      padding: const EdgeInsets.fromLTRB(24, 20, 24, 24),
      children: [
        SectionHeader(label: l10n.sectionAbout),
        GroupCard(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'LinkUnbound',
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const SizedBox(height: 4),
              Consumer(
                builder: (context, ref, _) {
                  final version =
                      ref.watch(packageInfoProvider).valueOrNull?.version ??
                      '…';
                  return Text(
                    l10n.appVersion(version),
                    style: Theme.of(context).textTheme.bodySmall,
                  );
                },
              ),
              const SizedBox(height: 8),
              Text(
                l10n.appDescription,
                style: Theme.of(context).textTheme.bodyMedium,
              ),
              const SizedBox(height: 4),
              Text(
                l10n.licenseName,
                style: Theme.of(context).textTheme.bodySmall,
              ),
            ],
          ),
        ),
        const SizedBox(height: 20),
        SectionHeader(label: l10n.sectionSupport),
        GroupCard(
          child: _LinkRow(
            icon: Icons.coffee_outlined,
            label: l10n.donateLabel,
            description: l10n.donateDescription,
            color: const Color(0xFFFFDD00),
            url: 'https://buymeacoffee.com/rgdevment',
          ),
        ),
        const SizedBox(height: 20),
        SectionHeader(label: l10n.sectionOtherTools),
        GroupCard(
          child: _LinkRow(
            iconWidget: ClipRRect(
              borderRadius: BorderRadius.circular(4),
              child: Image.asset(
                'assets/copypaste_icon.png',
                width: 20,
                height: 20,
              ),
            ),
            label: l10n.otherToolCopyPaste,
            description: l10n.otherToolCopyPasteDescription,
            color: colors.primary,
            url: 'https://github.com/rgdevment/CopyPaste',
          ),
        ),
      ],
    );
  }
}

class _LinkRow extends StatelessWidget {
  const _LinkRow({
    this.icon,
    this.iconWidget,
    required this.label,
    required this.description,
    required this.color,
    required this.url,
  });

  final IconData? icon;
  final Widget? iconWidget;
  final String label;
  final String description;
  final Color color;
  final String url;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: () => launchUrl(Uri.parse(url)),
      borderRadius: BorderRadius.circular(6),
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 10),
        child: Row(
          children: [
            if (iconWidget != null)
              iconWidget!
            else
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
            Icon(Icons.open_in_new, size: 16, color: color.withAlpha(120)),
          ],
        ),
      ),
    );
  }
}
