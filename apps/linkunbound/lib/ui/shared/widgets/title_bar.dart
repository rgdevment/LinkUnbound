import 'dart:io' show Platform;

import 'package:flutter/material.dart';
import 'package:window_manager/window_manager.dart';

class TitleBar extends StatelessWidget {
  const TitleBar({
    required this.tabController,
    required this.tabs,
    required this.onClose,
    super.key,
  });

  final TabController tabController;
  final List<String> tabs;
  final VoidCallback onClose;

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;
    final isMac = Platform.isMacOS;

    return GestureDetector(
      onPanStart: (_) => windowManager.startDragging(),
      child: Container(
        height: 40,
        color: colors.surface,
        child: Row(
          children: [
            // On macOS the native title bar (with the close button and app
            // title) sits above this row, so only the tabs are rendered here.
            const SizedBox(width: 12),
            if (!isMac) ...[
              Image.asset(
                'assets/app_icon.png',
                width: 16,
                height: 16,
                filterQuality: FilterQuality.medium,
              ),
              const SizedBox(width: 8),
              Text(
                'LinkUnbound',
                style: TextStyle(
                  fontSize: 12,
                  fontWeight: FontWeight.w600,
                  color: colors.onSurfaceVariant,
                ),
              ),
              const SizedBox(width: 24),
            ],
            Expanded(
              child: TabBar(
                controller: tabController,
                isScrollable: true,
                tabAlignment: TabAlignment.start,
                labelColor: colors.onSurface,
                unselectedLabelColor: colors.onSurfaceVariant,
                labelStyle: const TextStyle(
                  fontSize: 12,
                  fontWeight: FontWeight.w600,
                ),
                unselectedLabelStyle: const TextStyle(
                  fontSize: 12,
                  fontWeight: FontWeight.w400,
                ),
                indicatorSize: TabBarIndicatorSize.label,
                indicatorColor: colors.primary,
                indicatorWeight: 2,
                dividerHeight: 0,
                padding: EdgeInsets.zero,
                labelPadding: const EdgeInsets.symmetric(horizontal: 14),
                tabs: tabs.map((t) => Tab(height: 32, text: t)).toList(),
              ),
            ),
            if (!isMac) _CloseButton(onClose: onClose),
            if (isMac) const SizedBox(width: 12),
          ],
        ),
      ),
    );
  }
}

class _CloseButton extends StatefulWidget {
  const _CloseButton({required this.onClose});
  final VoidCallback onClose;

  @override
  State<_CloseButton> createState() => _CloseButtonState();
}

class _CloseButtonState extends State<_CloseButton> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onClose,
        child: Container(
          width: 46,
          height: 40,
          color: _hovered ? const Color(0xFFE81123) : Colors.transparent,
          alignment: Alignment.center,
          child: Icon(
            Icons.close,
            size: 16,
            color: _hovered
                ? Colors.white
                : Theme.of(context).colorScheme.onSurfaceVariant,
          ),
        ),
      ),
    );
  }
}
