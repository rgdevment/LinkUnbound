import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linkunbound_core/linkunbound_core.dart';
import 'package:url_launcher/url_launcher.dart';

import '../../l10n/app_localizations.dart';
import '../../platform/hotkey_service.dart';
import '../../providers.dart';
import '../shared/widgets/browser_tile.dart';
import '../shared/widgets/group_card.dart';
import '../shared/widgets/section_header.dart';

class GeneralPage extends ConsumerWidget {
  const GeneralPage({super.key});

  // On macOS LinkUnbound only registers as handler for http/https schemes
  // (Launch Services treats `public.html` separately and rarely surfaces it
  // to the user). The .htm/.html/.pdf extensions are Windows-only concepts
  // exposed via the registry.
  static const _allAssociations = [
    'http',
    'https',
    '.htm',
    '.html',
    '.xhtml',
    '.svg',
    '.pdf',
  ];
  static const _macAssociations = ['http', 'https'];

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final browsers = ref.watch(browsersProvider);
    final isDefaultAsync = ref.watch(isDefaultBrowserProvider);
    final isStartupAsync = ref.watch(isStartupEnabledProvider);
    final iconsDir = ref.read(iconsDirProvider);
    final edgeDismissed = ref.watch(edgeWarningDismissedProvider);
    final hasEdge = browsers.any(
      (b) => b.executablePath.toLowerCase().contains('msedge'),
    );

    return ListView(
      padding: const EdgeInsets.fromLTRB(24, 20, 24, 24),
      children: [
        ..._buildBrowsersSection(context, ref, browsers, iconsDir),
        const SizedBox(height: 20),
        ..._buildDefaultBrowserSection(
          context,
          isDefaultAsync,
          ref.watch(defaultAssociationsProvider),
        ),
        if (hasEdge && !edgeDismissed) ...[
          const SizedBox(height: 20),
          ..._buildEdgeWarningSection(context, ref),
        ],
        const SizedBox(height: 20),
        ..._buildStartupSection(context, ref, isStartupAsync),
        const SizedBox(height: 20),
        ..._buildAccessibilitySection(context, ref),
        const SizedBox(height: 20),
        ..._buildAppearanceSection(context, ref),
        const SizedBox(height: 20),
        ..._buildLanguageSection(context, ref),
      ],
    );
  }

  List<Widget> _buildDefaultBrowserSection(
    BuildContext context,
    AsyncValue<bool> isDefaultAsync,
    AsyncValue<Set<String>> associationsAsync,
  ) {
    final colors = Theme.of(context).colorScheme;
    final l10n = AppLocalizations.of(context)!;
    final isDefault = isDefaultAsync.valueOrNull == true;
    final associations = associationsAsync.valueOrNull ?? {};
    final assocList = Platform.isMacOS ? _macAssociations : _allAssociations;

    return [
      SectionHeader(label: l10n.sectionDefaultBrowser),
      GroupCard(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(
                  isDefault
                      ? Icons.check_circle_outline
                      : Icons.warning_amber_rounded,
                  size: 20,
                  color: isDefault ? Colors.green : colors.error,
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Text(
                    isDefault ? l10n.isDefaultBrowser : l10n.notDefaultBrowser,
                    style: Theme.of(context).textTheme.bodyMedium,
                  ),
                ),
                if (!isDefault)
                  TextButton(
                    onPressed: () => launchUrl(
                      Uri.parse(
                        Platform.isMacOS
                            ? 'x-apple.systempreferences:com.apple.preference.general'
                            : 'ms-settings:defaultapps?registeredAppUser=LinkUnbound',
                      ),
                    ),
                    child: Text(l10n.setDefault),
                  ),
              ],
            ),
            Padding(
              padding: const EdgeInsets.only(left: 32, top: 4),
              child: Text.rich(
                TextSpan(
                  children:
                      assocList
                          .map((a) {
                            final label = a.replaceAll('.', '').toUpperCase();
                            final active = associations.contains(a);
                            return TextSpan(
                              text: label,
                              style: TextStyle(
                                color: active
                                    ? colors.onSurfaceVariant
                                    : colors.onSurfaceVariant.withValues(
                                        alpha: 0.35,
                                      ),
                              ),
                            );
                          })
                          .expand(
                            (span) => [
                              span,
                              TextSpan(
                                text: ' · ',
                                style: TextStyle(
                                  color: colors.onSurfaceVariant.withValues(
                                    alpha: 0.35,
                                  ),
                                ),
                              ),
                            ],
                          )
                          .toList()
                        ..removeLast(),
                ),
                style: Theme.of(context).textTheme.bodySmall,
              ),
            ),
          ],
        ),
      ),
    ];
  }

  List<Widget> _buildEdgeWarningSection(BuildContext context, WidgetRef ref) {
    final colors = Theme.of(context).colorScheme;
    final l10n = AppLocalizations.of(context)!;
    final textTheme = Theme.of(context).textTheme;

    return [
      GroupCard(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(
                  Icons.warning_amber_rounded,
                  size: 20,
                  color: colors.primary,
                ),
                const SizedBox(width: 10),
                Expanded(
                  child: Text(
                    l10n.edgeWarningTitle,
                    style: textTheme.titleSmall?.copyWith(
                      color: colors.primary,
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            Text(l10n.edgeWarningBody, style: textTheme.bodySmall),
            const SizedBox(height: 10),
            Text(
              l10n.edgeWarningNote,
              style: textTheme.bodySmall?.copyWith(
                fontStyle: FontStyle.italic,
                color: colors.onSurfaceVariant,
              ),
            ),
            const SizedBox(height: 12),
            Align(
              alignment: Alignment.centerRight,
              child: TextButton(
                onPressed: () {
                  ref.read(edgeWarningDismissedProvider.notifier).dismiss();
                },
                child: Text(l10n.edgeWarningDismiss),
              ),
            ),
          ],
        ),
      ),
    ];
  }

  List<Widget> _buildStartupSection(
    BuildContext context,
    WidgetRef ref,
    AsyncValue<bool> isStartupAsync,
  ) {
    final l10n = AppLocalizations.of(context)!;

    return [
      SectionHeader(label: l10n.sectionStartup),
      GroupCard(
        child: Row(
          children: [
            Expanded(
              child: Text(
                l10n.launchAtStartup,
                style: Theme.of(context).textTheme.bodyMedium,
              ),
            ),
            Switch(
              value: isStartupAsync.valueOrNull ?? false,
              onChanged: (enabled) async {
                final messenger = ScaffoldMessenger.maybeOf(context);
                final errorMsg = l10n.errorStartupToggle;
                final service = ref.read(startupServiceProvider);
                try {
                  if (enabled) {
                    await service.enable(Platform.resolvedExecutable);
                  } else {
                    await service.disable();
                  }
                } on Object {
                  messenger?.showSnackBar(SnackBar(content: Text(errorMsg)));
                } finally {
                  ref.invalidate(isStartupEnabledProvider);
                }
              },
            ),
          ],
        ),
      ),
    ];
  }

  List<Widget> _buildAccessibilitySection(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final hotkey = ref.watch(globalHotkeyProvider);
    final hideTray = ref.watch(hideTrayProvider);
    final hasHotkey = hotkey != null && hotkey.isNotEmpty;

    final hideTraySubtitle = !hasHotkey
        ? l10n.hideTraySubtitleNoHotkey
        : Platform.isMacOS
        ? l10n.hideTraySubtitleMac
        : l10n.hideTraySubtitleWindows;

    return [
      SectionHeader(label: l10n.sectionAccessibility),
      GroupCard(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Expanded(
                  child: Text(
                    l10n.globalHotkeyLabel,
                    style: Theme.of(context).textTheme.bodyMedium,
                  ),
                ),
                DropdownButtonHideUnderline(
                  child: DropdownButton<String?>(
                    value: hotkey,
                    isDense: true,
                    dropdownColor: Theme.of(context).colorScheme.surfaceBright,
                    style: Theme.of(context).textTheme.bodyMedium,
                    items: [
                      DropdownMenuItem<String?>(
                        value: null,
                        child: Text(l10n.globalHotkeyNone),
                      ),
                      // Guard against a saved hotkey from another OS that isn't
                      // in the current platform's preset list.
                      if (hotkey != null &&
                          HotkeyPreset.defaults.every(
                            (p) => p.serialized != hotkey,
                          ))
                        DropdownMenuItem<String?>(
                          value: hotkey,
                          child: Text(hotkey),
                        ),
                      for (final preset in HotkeyPreset.defaults)
                        DropdownMenuItem<String?>(
                          value: preset.serialized,
                          child: Text(preset.label),
                        ),
                    ],
                    onChanged: (value) {
                      ref.read(globalHotkeyProvider.notifier).setHotkey(value);
                    },
                  ),
                ),
              ],
            ),
            const Divider(height: 16),
            Row(
              children: [
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        l10n.hideTrayLabel,
                        style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                          color: hasHotkey
                              ? null
                              : Theme.of(
                                  context,
                                ).colorScheme.onSurface.withValues(alpha: 0.38),
                        ),
                      ),
                      Text(
                        hideTraySubtitle,
                        style: Theme.of(context).textTheme.bodySmall?.copyWith(
                          color: Theme.of(context).colorScheme.onSurfaceVariant,
                        ),
                      ),
                    ],
                  ),
                ),
                Switch(
                  // Safeguard: hide-tray is disabled unless a hotkey is set,
                  // preventing the app from becoming unreachable.
                  value: hasHotkey && hideTray,
                  onChanged: hasHotkey
                      ? (enabled) {
                          ref
                              .read(hideTrayProvider.notifier)
                              .setHideTray(enabled);
                        }
                      : null,
                ),
              ],
            ),
          ],
        ),
      ),
    ];
  }

  List<Widget> _buildAppearanceSection(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final themeMode = ref.watch(themeModeProvider);

    return [
      SectionHeader(label: l10n.sectionAppearance),
      GroupCard(
        child: Row(
          children: [
            Expanded(
              child: Text(
                l10n.themeLabel,
                style: Theme.of(context).textTheme.bodyMedium,
              ),
            ),
            DropdownButtonHideUnderline(
              child: DropdownButton<ThemeMode>(
                value: themeMode,
                isDense: true,
                dropdownColor: Theme.of(context).colorScheme.surfaceBright,
                style: Theme.of(context).textTheme.bodyMedium,
                items: [
                  DropdownMenuItem(
                    value: ThemeMode.system,
                    child: Text(l10n.themeSystem),
                  ),
                  DropdownMenuItem(
                    value: ThemeMode.light,
                    child: Text(l10n.themeLight),
                  ),
                  DropdownMenuItem(
                    value: ThemeMode.dark,
                    child: Text(l10n.themeDark),
                  ),
                ],
                onChanged: (mode) {
                  if (mode == null) return;
                  ref.read(themeModeProvider.notifier).setThemeMode(mode);
                },
              ),
            ),
          ],
        ),
      ),
    ];
  }

  List<Widget> _buildLanguageSection(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final locale = ref.watch(localeProvider);

    return [
      SectionHeader(label: l10n.sectionLanguage),
      GroupCard(
        child: Row(
          children: [
            Expanded(
              child: Text(
                l10n.sectionLanguage,
                style: Theme.of(context).textTheme.bodyMedium,
              ),
            ),
            DropdownButtonHideUnderline(
              child: DropdownButton<String>(
                value: locale?.languageCode ?? 'auto',
                isDense: true,
                dropdownColor: Theme.of(context).colorScheme.surfaceBright,
                style: Theme.of(context).textTheme.bodyMedium,
                items: [
                  DropdownMenuItem(
                    value: 'auto',
                    child: Text(l10n.languageAuto),
                  ),
                  DropdownMenuItem(
                    value: 'en',
                    child: Text(l10n.languageEnglish),
                  ),
                  DropdownMenuItem(
                    value: 'es',
                    child: Text(l10n.languageSpanish),
                  ),
                ],
                onChanged: (code) {
                  final newLocale = code == null || code == 'auto'
                      ? null
                      : Locale(code);
                  ref.read(localeProvider.notifier).setLocale(newLocale);
                },
              ),
            ),
          ],
        ),
      ),
    ];
  }

  List<Widget> _buildBrowsersSection(
    BuildContext context,
    WidgetRef ref,
    List<Browser> browsers,
    Directory iconsDir,
  ) {
    final colors = Theme.of(context).colorScheme;
    final l10n = AppLocalizations.of(context)!;

    return [
      SectionHeader(
        label: l10n.sectionBrowsers,
        trailing: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            IconButton(
              onPressed: () => _showAddBrowserDialog(context, ref),
              icon: const Icon(Icons.add, size: 18),
              padding: EdgeInsets.zero,
              constraints: const BoxConstraints(minWidth: 28, minHeight: 28),
              tooltip: l10n.addBrowserTooltip,
            ),
            const SizedBox(width: 4),
            IconButton(
              onPressed: () => _refreshBrowsers(context, ref, iconsDir),
              icon: const Icon(Icons.refresh, size: 18),
              padding: EdgeInsets.zero,
              constraints: const BoxConstraints(minWidth: 28, minHeight: 28),
              tooltip: l10n.refreshBrowsersTooltip,
            ),
          ],
        ),
      ),
      GroupCard(
        padding: const EdgeInsets.symmetric(vertical: 4, horizontal: 4),
        child: SizedBox(
          height: 5 * 44.0,
          child: ReorderableListView.builder(
            buildDefaultDragHandles: false,
            itemCount: browsers.length,
            onReorderItem: (oldIndex, newIndex) {
              ref.read(browsersProvider.notifier).reorder(oldIndex, newIndex);
            },
            proxyDecorator: (child, index, animation) {
              return AnimatedBuilder(
                animation: animation,
                builder: (context, child) => Material(
                  color: colors.surfaceBright,
                  borderRadius: BorderRadius.circular(6),
                  elevation: 4,
                  child: child,
                ),
                child: child,
              );
            },
            itemBuilder: (context, index) {
              final b = browsers[index];
              final isEdge = b.executablePath.toLowerCase().contains('msedge');
              return BrowserTile(
                key: ValueKey(b.id),
                name: b.name,
                iconPath:
                    '${iconsDir.path}${Platform.pathSeparator}${b.id}.png',
                onTap: () => _showEditBrowserDialog(context, ref, b),
                trailing: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    if (isEdge)
                      Tooltip(
                        message: l10n.edgeWarningBody,
                        child: Icon(
                          Icons.warning_amber_rounded,
                          size: 16,
                          color: colors.error,
                        ),
                      ),
                    PopupMenuButton<String>(
                      onSelected: (action) async {
                        switch (action) {
                          case 'edit':
                            _showEditBrowserDialog(context, ref, b);
                          case 'duplicate':
                            await _duplicateBrowser(context, ref, b);
                          case 'remove':
                            ref.read(browsersProvider.notifier).remove(b.id);
                        }
                      },
                      icon: Icon(
                        Icons.more_vert,
                        size: 16,
                        color: colors.onSurfaceVariant,
                      ),
                      padding: EdgeInsets.zero,
                      constraints: const BoxConstraints(
                        minWidth: 28,
                        minHeight: 28,
                      ),
                      itemBuilder: (_) => [
                        PopupMenuItem(
                          value: 'edit',
                          child: Text(l10n.menuEdit),
                        ),
                        PopupMenuItem(
                          value: 'duplicate',
                          child: Text(l10n.menuDuplicate),
                        ),
                        PopupMenuItem(
                          value: 'remove',
                          child: Text(l10n.menuRemove),
                        ),
                      ],
                    ),
                    ReorderableDragStartListener(
                      index: index,
                      child: Icon(
                        Icons.drag_handle,
                        size: 18,
                        color: colors.onSurfaceVariant,
                      ),
                    ),
                  ],
                ),
              );
            },
          ),
        ),
      ),
    ];
  }

  Future<void> _refreshBrowsers(
    BuildContext context,
    WidgetRef ref,
    Directory iconsDir,
  ) async {
    final result = await ref.read(browsersProvider.notifier).refresh();
    final iconExtractor = ref.read(iconExtractorProvider);
    for (final browser in ref.read(browsersProvider)) {
      try {
        await iconExtractor.extractIcon(
          browser.executablePath,
          '${iconsDir.path}${Platform.pathSeparator}${browser.id}.png',
        );
      } on Exception {
        // Best-effort icon extraction
      }
    }
    ref.invalidate(browsersProvider);
    if (context.mounted) {
      final l10n = AppLocalizations.of(context)!;
      final message = result.added > 0 || result.removed > 0
          ? l10n.refreshResult(result.added, result.removed)
          : l10n.refreshNoChanges;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(message),
          duration: const Duration(seconds: 2),
          behavior: SnackBarBehavior.floating,
          width: 280,
        ),
      );
    }
  }

  void _showAddBrowserDialog(BuildContext context, WidgetRef ref) {
    _showBrowserFormDialog(context, ref);
  }

  void _showEditBrowserDialog(
    BuildContext context,
    WidgetRef ref,
    Browser browser,
  ) {
    _showBrowserFormDialog(context, ref, existing: browser);
  }

  Future<void> _duplicateBrowser(
    BuildContext context,
    WidgetRef ref,
    Browser source,
  ) async {
    final copyId =
        'custom-${source.id}-copy-${DateTime.now().millisecondsSinceEpoch}';
    final copy = Browser(
      id: copyId,
      name: '${source.name} (Copy)',
      executablePath: source.executablePath,
      iconPath: source.iconPath,
      extraArgs: [...source.extraArgs],
      isCustom: true,
    );
    await ref.read(browsersProvider.notifier).add(copy);

    final iconsDir = ref.read(iconsDirProvider);
    final sourceIcon = File(
      '${iconsDir.path}${Platform.pathSeparator}${source.id}.png',
    );
    final destIcon = File(
      '${iconsDir.path}${Platform.pathSeparator}$copyId.png',
    );
    if (sourceIcon.existsSync()) {
      await sourceIcon.copy(destIcon.path);
    }
  }

  void _showBrowserFormDialog(
    BuildContext context,
    WidgetRef ref, {
    Browser? existing,
  }) {
    final isEdit = existing != null;
    final nameController = TextEditingController(text: existing?.name ?? '');
    final pathController = TextEditingController(
      text: existing?.executablePath ?? '',
    );
    final argsController = TextEditingController(
      text: existing?.extraArgs.join(' ') ?? '',
    );
    final iconController = TextEditingController();

    showDialog<void>(
      context: context,
      builder: (ctx) {
        final colors = Theme.of(ctx).colorScheme;
        final l10n = AppLocalizations.of(ctx)!;

        return Dialog(
          backgroundColor: colors.surfaceContainer,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(12),
          ),
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 420),
            child: Padding(
              padding: const EdgeInsets.all(24),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    isEdit ? l10n.editBrowserTitle : l10n.addBrowserTitle,
                    style: Theme.of(ctx).textTheme.titleMedium,
                  ),
                  const SizedBox(height: 16),
                  _FormField(controller: nameController, label: l10n.fieldName),
                  const SizedBox(height: 12),
                  _FormField(
                    controller: pathController,
                    label: l10n.fieldExecutablePath,
                    enabled: isEdit ? existing.isCustom : true,
                  ),
                  const SizedBox(height: 12),
                  _FormField(
                    controller: argsController,
                    label: l10n.fieldExtraArgs,
                  ),
                  const SizedBox(height: 12),
                  _FormField(
                    controller: iconController,
                    label: l10n.fieldIconPath,
                    hint: l10n.fieldIconHint,
                  ),
                  const SizedBox(height: 20),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.end,
                    children: [
                      TextButton(
                        onPressed: () => Navigator.of(ctx).pop(),
                        child: Text(l10n.cancel),
                      ),
                      const SizedBox(width: 8),
                      FilledButton(
                        onPressed: () => _saveBrowser(
                          ctx,
                          ref,
                          existing: existing,
                          name: nameController.text.trim(),
                          path: pathController.text.trim(),
                          args: argsController.text.trim(),
                          customIcon: iconController.text.trim(),
                        ),
                        child: Text(isEdit ? l10n.save : l10n.add),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ),
        );
      },
    );
  }

  Future<void> _saveBrowser(
    BuildContext ctx,
    WidgetRef ref, {
    Browser? existing,
    required String name,
    required String path,
    required String args,
    required String customIcon,
  }) async {
    if (name.isEmpty || path.isEmpty) return;

    final extraArgs = args.isEmpty ? <String>[] : args.split(RegExp(r'\s+'));

    final String browserId;
    if (existing != null) {
      final updated = existing.copyWith(
        name: name,
        executablePath: existing.isCustom ? path : null,
        extraArgs: extraArgs,
      );
      await ref.read(browsersProvider.notifier).update(existing.id, updated);
      browserId = existing.id;
    } else {
      final id = name
          .toLowerCase()
          .replaceAll(RegExp(r'[^a-z0-9]+'), '-')
          .replaceAll(RegExp(r'^-|-$'), '');
      browserId = 'custom-$id';
      await ref
          .read(browsersProvider.notifier)
          .add(
            Browser(
              id: browserId,
              name: name,
              executablePath: path,
              iconPath: path,
              isCustom: true,
              extraArgs: extraArgs,
            ),
          );
    }

    await _updateBrowserIcon(ref, browserId, customIcon, path);
    if (ctx.mounted) Navigator.of(ctx).pop();
  }

  Future<void> _updateBrowserIcon(
    WidgetRef ref,
    String browserId,
    String customIcon,
    String exePath,
  ) async {
    final iconsDir = ref.read(iconsDirProvider);
    final iconSource = customIcon.isNotEmpty ? customIcon : exePath;
    final iconDest = File(
      '${iconsDir.path}${Platform.pathSeparator}$browserId.png',
    );

    if (customIcon.isNotEmpty && iconDest.existsSync()) {
      await iconDest.delete();
    }

    try {
      await ref
          .read(iconExtractorProvider)
          .extractIcon(
            iconSource,
            '${iconsDir.path}${Platform.pathSeparator}$browserId.png',
          );
    } on Exception {
      // Best-effort
    }
  }
}

class _FormField extends StatelessWidget {
  const _FormField({
    required this.controller,
    required this.label,
    this.hint,
    this.enabled = true,
  });

  final TextEditingController controller;
  final String label;
  final String? hint;
  final bool enabled;

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;

    return TextField(
      controller: controller,
      enabled: enabled,
      decoration: InputDecoration(
        labelText: label,
        hintText: hint,
        filled: true,
        fillColor: colors.surfaceBright,
        border: OutlineInputBorder(borderRadius: BorderRadius.circular(6)),
        contentPadding: const EdgeInsets.symmetric(
          horizontal: 12,
          vertical: 10,
        ),
      ),
      style: Theme.of(context).textTheme.bodyMedium,
    );
  }
}
