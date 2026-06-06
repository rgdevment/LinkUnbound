import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linkunbound_core/linkunbound_core.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:window_manager/window_manager.dart';

import '../../l10n/app_localizations.dart';
import '../../platform/windows/win_package_context.dart';
import '../../providers.dart';
import '../shared/widgets/title_bar.dart';
import 'about_page.dart';
import 'general_page.dart';
import 'maintenance_page.dart';
import 'rules_page.dart';

class SettingsView extends ConsumerStatefulWidget {
  const SettingsView({super.key});

  @override
  ConsumerState<SettingsView> createState() => _SettingsViewState();
}

class _SettingsViewState extends ConsumerState<SettingsView>
    with SingleTickerProviderStateMixin, WindowListener {
  late final TabController _tabController;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 4, vsync: this);
    windowManager.addListener(this);
  }

  @override
  void dispose() {
    windowManager.removeListener(this);
    _tabController.dispose();
    super.dispose();
  }

  @override
  void onWindowFocus() {
    ref.invalidate(isDefaultBrowserProvider);
    ref.invalidate(defaultAssociationsProvider);
  }

  Future<void> _hideWindow() async {
    await windowManager.hide();
    ref.read(appStateProvider.notifier).hide();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final updateAsync = ref.watch(updateInfoProvider);

    return Column(
      children: [
        TitleBar(
          tabController: _tabController,
          tabs: [
            l10n.tabGeneral,
            l10n.tabRules,
            l10n.tabMaintenance,
            l10n.tabAbout,
          ],
          onClose: _hideWindow,
        ),
        Divider(
          height: 0.5,
          color: Theme.of(context).colorScheme.outline.withAlpha(60),
        ),
        Expanded(
          child: TabBarView(
            controller: _tabController,
            physics: const NeverScrollableScrollPhysics(),
            children: const [
              GeneralPage(),
              RulesPage(),
              MaintenancePage(),
              AboutPage(),
            ],
          ),
        ),
        if (updateAsync.valueOrNull case final update?)
          _UpdateBanner(update: update, l10n: l10n),
      ],
    );
  }
}

class _UpdateBanner extends StatelessWidget {
  const _UpdateBanner({required this.update, required this.l10n});

  final UpdateInfo update;
  final AppLocalizations l10n;

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;
    final inStore = isRunningInMsix();
    final message = inStore
        ? l10n.updateAvailableStore(update.latestVersion)
        : l10n.updateAvailable(update.latestVersion);

    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      decoration: BoxDecoration(
        color: colors.primary.withAlpha(20),
        border: Border(top: BorderSide(color: colors.primary.withAlpha(40))),
      ),
      child: Row(
        children: [
          Icon(Icons.upgrade_rounded, size: 16, color: colors.primary),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              message,
              style: Theme.of(
                context,
              ).textTheme.bodySmall?.copyWith(color: colors.primary),
            ),
          ),
          if (!inStore)
            TextButton(
              style: TextButton.styleFrom(
                padding: const EdgeInsets.symmetric(horizontal: 12),
                minimumSize: const Size(0, 28),
                tapTargetSize: MaterialTapTargetSize.shrinkWrap,
              ),
              onPressed: () => launchUrl(Uri.parse(update.releaseUrl)),
              child: Text(
                l10n.updateDownload,
                style: TextStyle(fontSize: 12, color: colors.primary),
              ),
            ),
        ],
      ),
    );
  }
}
