import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linkunbound_core/linkunbound_core.dart';
import 'package:logging/logging.dart';

import '../../l10n/app_localizations.dart';
import '../../providers.dart';

final _log = Logger('PickerView');

class PickerView extends ConsumerStatefulWidget {
  const PickerView({required this.url, super.key});

  final String url;

  @override
  ConsumerState<PickerView> createState() => _PickerViewState();
}

class _PickerViewState extends ConsumerState<PickerView> {
  bool _alwaysOpen = false;

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;
    final browsers = ref.watch(browsersProvider);
    final iconsDir = ref.read(iconsDirProvider);
    final uri = Uri.tryParse(widget.url);
    final isLocalFile = uri?.scheme == 'file';
    // For local files show just the filename in the bold line — the full
    // path is privacy-sensitive (it can leak `$HOME`/project paths) and is
    // already redacted in logs. Hover the row to copy the URL if needed.
    final domain = isLocalFile
        ? (uri!.pathSegments.isNotEmpty ? uri.pathSegments.last : widget.url)
        : (uri?.host ?? widget.url);

    return Focus(
      autofocus: true,
      onKeyEvent: (node, event) {
        if (event is KeyDownEvent &&
            event.logicalKey == LogicalKeyboardKey.escape) {
          ref.read(appStateProvider.notifier).hide();
          return KeyEventResult.handled;
        }

        if (event is KeyDownEvent) {
          final index = _keyToIndex(event.logicalKey);
          if (index != null && index < browsers.length) {
            _launch(browsers[index], iconsDir);
            return KeyEventResult.handled;
          }
        }

        return KeyEventResult.ignored;
      },
      child: Column(
        children: [
          _UrlHeader(url: widget.url, domain: domain, isLocalFile: isLocalFile),
          Divider(height: 0.5, color: colors.outline.withAlpha(50)),
          Expanded(
            // Scrollbar appears when browsers > maxVisible (6); shortcuts 1-9
            // still work for off-screen rows — the scrollbar signals that.
            child: Scrollbar(
              thumbVisibility: browsers.length > 6,
              child: ListView.builder(
                padding: const EdgeInsets.symmetric(vertical: 4),
                itemCount: browsers.length,
                itemBuilder: (context, index) => _BrowserRow(
                  browser: browsers[index],
                  iconPath: '${iconsDir.path}/${browsers[index].id}.png',
                  shortcut: index < 9 ? '${index + 1}' : null,
                  onTap: () => _launch(browsers[index], iconsDir),
                ),
              ),
            ),
          ),
          Divider(height: 0.5, color: colors.outline.withAlpha(50)),
          _AlwaysOpenFooter(
            value: _alwaysOpen,
            onChanged: (v) => setState(() => _alwaysOpen = v),
          ),
        ],
      ),
    );
  }

  void _launch(Browser browser, Directory iconsDir) {
    final launchService = ref.read(launchServiceProvider);
    launchService.launch(browser.executablePath, widget.url, browser.extraArgs);

    if (_alwaysOpen) {
      final ruleService = ref.read(ruleServiceProvider);
      final uri = Uri.tryParse(widget.url);
      if (uri != null && uri.host.isNotEmpty) {
        ruleService.addRule(Rule(domain: uri.host, browserId: browser.id));
        unawaited(
          ruleService.save().catchError((Object e, StackTrace st) {
            _log.warning('Failed to persist always-open rule', e, st);
          }),
        );
        ref.invalidate(rulesProvider);
      }
    }

    ref.read(appStateProvider.notifier).hide();
  }

  int? _keyToIndex(LogicalKeyboardKey key) => switch (key) {
    LogicalKeyboardKey.digit1 || LogicalKeyboardKey.numpad1 => 0,
    LogicalKeyboardKey.digit2 || LogicalKeyboardKey.numpad2 => 1,
    LogicalKeyboardKey.digit3 || LogicalKeyboardKey.numpad3 => 2,
    LogicalKeyboardKey.digit4 || LogicalKeyboardKey.numpad4 => 3,
    LogicalKeyboardKey.digit5 || LogicalKeyboardKey.numpad5 => 4,
    LogicalKeyboardKey.digit6 || LogicalKeyboardKey.numpad6 => 5,
    LogicalKeyboardKey.digit7 || LogicalKeyboardKey.numpad7 => 6,
    LogicalKeyboardKey.digit8 || LogicalKeyboardKey.numpad8 => 7,
    LogicalKeyboardKey.digit9 || LogicalKeyboardKey.numpad9 => 8,
    _ => null,
  };
}

class _UrlHeader extends StatelessWidget {
  const _UrlHeader({
    required this.url,
    required this.domain,
    required this.isLocalFile,
  });
  final String url;
  final String domain;
  final bool isLocalFile;

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;
    // For local files, show only the parent directory in the secondary line
    // (`…/parent/file.html`) instead of the full path — same redaction rule
    // we use for logs. Hover on the copy button to copy the full URL.
    final secondary = isLocalFile ? _redactedFilePath(url) : url;

    return Padding(
      padding: const EdgeInsets.fromLTRB(14, 12, 8, 10),
      child: Row(
        children: [
          Icon(
            isLocalFile ? Icons.insert_drive_file_outlined : Icons.link,
            size: 16,
            color: colors.primary,
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  domain,
                  style: TextStyle(
                    fontSize: 13,
                    fontWeight: FontWeight.w600,
                    color: colors.onSurface,
                  ),
                  overflow: TextOverflow.ellipsis,
                ),
                Text(
                  secondary,
                  style: TextStyle(
                    fontSize: 11,
                    color: colors.onSurfaceVariant,
                  ),
                  overflow: TextOverflow.ellipsis,
                  maxLines: 1,
                ),
              ],
            ),
          ),
          IconButton(
            onPressed: () => Clipboard.setData(ClipboardData(text: url)),
            icon: Icon(Icons.copy, size: 14, color: colors.onSurfaceVariant),
            padding: EdgeInsets.zero,
            constraints: const BoxConstraints(minWidth: 32, minHeight: 32),
            tooltip: AppLocalizations.of(context)!.copyUrl,
          ),
        ],
      ),
    );
  }
}

/// Returns `…/parent/file.html` for a `file://` URL — strips `$HOME` and
/// project paths from the visible UI.
String _redactedFilePath(String fileUrl) {
  final uri = Uri.tryParse(fileUrl);
  if (uri == null) return fileUrl;
  final segs = uri.pathSegments.where((s) => s.isNotEmpty).toList();
  if (segs.isEmpty) return fileUrl;
  if (segs.length == 1) return '…/${segs.last}';
  return '…/${segs[segs.length - 2]}/${segs.last}';
}

/// Renders the browser icon using Image.file + errorBuilder so existsSync
/// is never called in build — async image loading handles missing files.
class _BrowserIcon extends StatelessWidget {
  const _BrowserIcon({required this.iconPath, required this.fallbackColor});

  final String iconPath;
  final Color fallbackColor;

  @override
  Widget build(BuildContext context) {
    if (iconPath.isEmpty) {
      return Icon(Icons.public, size: 28, color: fallbackColor);
    }
    // cacheWidth=56 (2× render size) covers 2× DPI; the codec downsamples
    // on decode instead of at paint time, saving texture memory.
    return Image.file(
      File(iconPath),
      cacheWidth: 56,
      filterQuality: FilterQuality.medium,
      errorBuilder: (context, error, stack) =>
          Icon(Icons.public, size: 28, color: fallbackColor),
    );
  }
}

class _BrowserRow extends StatefulWidget {
  const _BrowserRow({
    required this.browser,
    required this.iconPath,
    required this.onTap,
    this.shortcut,
  });

  final Browser browser;
  final String iconPath;
  final VoidCallback onTap;
  final String? shortcut;

  @override
  State<_BrowserRow> createState() => _BrowserRowState();
}

class _BrowserRowState extends State<_BrowserRow> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;

    return MouseRegion(
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 100),
          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
          color: _hovered ? colors.surfaceBright : Colors.transparent,
          child: Row(
            children: [
              SizedBox(
                width: 28,
                height: 28,
                child: _BrowserIcon(
                  iconPath: widget.iconPath,
                  fallbackColor: colors.onSurfaceVariant,
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Text(
                  widget.browser.name,
                  style: TextStyle(fontSize: 13, color: colors.onSurface),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              if (widget.shortcut != null)
                Container(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 6,
                    vertical: 2,
                  ),
                  decoration: BoxDecoration(
                    color: colors.surfaceBright,
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: Text(
                    widget.shortcut!,
                    style: TextStyle(
                      fontSize: 11,
                      fontWeight: FontWeight.w600,
                      color: colors.onSurfaceVariant,
                    ),
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }
}

class _AlwaysOpenFooter extends ConsumerWidget {
  const _AlwaysOpenFooter({required this.value, required this.onChanged});
  final bool value;
  final ValueChanged<bool> onChanged;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final colors = Theme.of(context).colorScheme;
    final hasUpdate = ref.watch(updateInfoProvider).valueOrNull != null;

    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
      child: Row(
        children: [
          SizedBox(
            width: 18,
            height: 18,
            child: Checkbox(
              value: value,
              onChanged: (v) => onChanged(v ?? false),
              activeColor: colors.primary,
              side: BorderSide(color: colors.outline),
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(3),
              ),
              materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
            ),
          ),
          const SizedBox(width: 8),
          Text(
            AppLocalizations.of(context)!.alwaysOpenHere,
            style: TextStyle(fontSize: 12, color: colors.onSurfaceVariant),
          ),
          const Spacer(),
          if (hasUpdate) const _UpdateDot(),
        ],
      ),
    );
  }
}

class _UpdateDot extends StatefulWidget {
  const _UpdateDot();

  @override
  State<_UpdateDot> createState() => _UpdateDotState();
}

class _UpdateDotState extends State<_UpdateDot>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1200),
    )..repeat(reverse: true);
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;

    return Tooltip(
      message: AppLocalizations.of(context)!.updateTooltip,
      preferBelow: false,
      child: FadeTransition(
        opacity: Tween<double>(begin: 0.3, end: 1.0).animate(
          CurvedAnimation(parent: _controller, curve: Curves.easeInOut),
        ),
        child: Container(
          width: 8,
          height: 8,
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            color: colors.primary,
          ),
        ),
      ),
    );
  }
}
