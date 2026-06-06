import 'dart:io';

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:linkunbound/ui/shared/widgets/base_dialog.dart';
import 'package:linkunbound/ui/shared/widgets/browser_tile.dart';
import 'package:linkunbound/ui/shared/widgets/group_card.dart';
import 'package:linkunbound/ui/shared/widgets/section_header.dart';
import 'package:linkunbound/ui/shared/widgets/title_bar.dart';

import '../helpers.dart';

void main() {
  group('BaseDialog', () {
    testWidgets('renders title and content', (tester) async {
      await tester.pumpWidget(
        buildTestApp(
          const BaseDialog(title: 'My Title', content: 'My Content'),
          overrides: [],
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('My Title'), findsOneWidget);
      expect(find.text('My Content'), findsOneWidget);
    });

    testWidgets('shows Cancel button', (tester) async {
      await tester.pumpWidget(
        buildTestApp(
          const BaseDialog(title: 'T', content: 'C'),
          overrides: [],
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('Cancel'), findsOneWidget);
    });

    testWidgets('shows default Confirm label when confirmLabel is null', (
      tester,
    ) async {
      await tester.pumpWidget(
        buildTestApp(
          const BaseDialog(title: 'T', content: 'C'),
          overrides: [],
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('Confirm'), findsOneWidget);
    });

    testWidgets('shows custom confirmLabel', (tester) async {
      await tester.pumpWidget(
        buildTestApp(
          const BaseDialog(title: 'T', content: 'C', confirmLabel: 'Delete'),
          overrides: [],
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('Delete'), findsOneWidget);
    });

    testWidgets('cancel button triggers onCancel callback', (tester) async {
      var cancelled = false;
      await tester.pumpWidget(
        buildTestApp(
          BaseDialog(
            title: 'T',
            content: 'C',
            onCancel: () => cancelled = true,
          ),
          overrides: [],
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Cancel'));
      await tester.pumpAndSettle();
      expect(cancelled, isTrue);
    });

    testWidgets('confirm button triggers onConfirm callback', (tester) async {
      var confirmed = false;
      await tester.pumpWidget(
        buildTestApp(
          BaseDialog(
            title: 'T',
            content: 'C',
            onConfirm: () => confirmed = true,
          ),
          overrides: [],
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Confirm'));
      await tester.pumpAndSettle();
      expect(confirmed, isTrue);
    });

    testWidgets('renders with confirmColor', (tester) async {
      await tester.pumpWidget(
        buildTestApp(
          const BaseDialog(
            title: 'T',
            content: 'C',
            confirmLabel: 'Delete',
            confirmColor: Colors.red,
          ),
          overrides: [],
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('Delete'), findsOneWidget);
    });

    testWidgets('default cancel dismisses navigator', (tester) async {
      await tester.pumpWidget(
        buildTestApp(
          Builder(
            builder: (ctx) => TextButton(
              onPressed: () => showDialog<void>(
                context: ctx,
                builder: (_) => const BaseDialog(title: 'T', content: 'C'),
              ),
              child: const Text('Open'),
            ),
          ),
          overrides: [],
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Open'));
      await tester.pumpAndSettle();
      expect(find.byType(BaseDialog), findsOneWidget);
      await tester.tap(find.text('Cancel'));
      await tester.pumpAndSettle();
      expect(find.byType(BaseDialog), findsNothing);
    });
  });

  group('GroupCard', () {
    testWidgets('renders its child', (tester) async {
      await tester.pumpWidget(
        buildTestApp(const GroupCard(child: Text('Hello')), overrides: []),
      );
      await tester.pumpAndSettle();
      expect(find.text('Hello'), findsOneWidget);
    });

    testWidgets('renders with custom padding', (tester) async {
      await tester.pumpWidget(
        buildTestApp(
          const GroupCard(padding: EdgeInsets.all(4), child: Text('Padded')),
          overrides: [],
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('Padded'), findsOneWidget);
    });

    testWidgets('renders with default padding when padding is null', (
      tester,
    ) async {
      await tester.pumpWidget(
        buildTestApp(const GroupCard(child: Text('Default')), overrides: []),
      );
      await tester.pumpAndSettle();
      final container = tester.widget<Container>(find.byType(Container).first);
      expect(container.padding, const EdgeInsets.all(14));
    });
  });

  group('SectionHeader', () {
    testWidgets('renders label text', (tester) async {
      await tester.pumpWidget(
        buildTestApp(const SectionHeader(label: 'SETTINGS'), overrides: []),
      );
      await tester.pumpAndSettle();
      expect(find.text('SETTINGS'), findsOneWidget);
    });

    testWidgets('renders trailing widget when provided', (tester) async {
      await tester.pumpWidget(
        buildTestApp(
          const SectionHeader(label: 'SETTINGS', trailing: Icon(Icons.add)),
          overrides: [],
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byIcon(Icons.add), findsOneWidget);
    });

    testWidgets('renders without trailing when not provided', (tester) async {
      await tester.pumpWidget(
        buildTestApp(const SectionHeader(label: 'SETTINGS'), overrides: []),
      );
      await tester.pumpAndSettle();
      expect(find.byIcon(Icons.add), findsNothing);
    });
  });

  group('BrowserTile', () {
    testWidgets('renders browser name', (tester) async {
      await tester.pumpWidget(
        buildTestApp(
          const BrowserTile(
            name: 'Google Chrome',
            iconPath: '/nonexistent/icon.png',
          ),
          overrides: [],
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('Google Chrome'), findsOneWidget);
    });

    testWidgets('shows fallback icon when icon file does not exist', (
      tester,
    ) async {
      await tester.pumpWidget(
        buildTestApp(
          const BrowserTile(name: 'Chrome', iconPath: '/nonexistent/icon.png'),
          overrides: [],
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byIcon(Icons.public), findsOneWidget);
    });

    testWidgets('renders trailing widget when provided', (tester) async {
      await tester.pumpWidget(
        buildTestApp(
          const BrowserTile(
            name: 'Chrome',
            iconPath: '/nonexistent/icon.png',
            trailing: Icon(Icons.check),
          ),
          overrides: [],
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byIcon(Icons.check), findsOneWidget);
    });

    testWidgets('calls onTap when tapped', (tester) async {
      var tapped = false;
      await tester.pumpWidget(
        buildTestApp(
          BrowserTile(
            name: 'Chrome',
            iconPath: '/nonexistent/icon.png',
            onTap: () => tapped = true,
          ),
          overrides: [],
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byType(BrowserTile));
      expect(tapped, isTrue);
    });

    testWidgets('shows Image.file when icon file exists', (tester) async {
      final file = File.fromUri(
        Uri.file(
          '${Directory.systemTemp.path}/test_icon_${DateTime.now().millisecondsSinceEpoch}.png',
        ),
      );
      // Create a minimal valid file so existsSync returns true
      file.writeAsBytesSync([0x89, 0x50, 0x4E, 0x47]);
      addTearDown(file.deleteSync);

      await tester.pumpWidget(
        buildTestApp(
          BrowserTile(name: 'Chrome', iconPath: file.path),
          overrides: [],
        ),
      );
      await tester.pump();
      expect(find.byIcon(Icons.public), findsNothing);
      expect(find.byType(Image), findsOneWidget);
    });
  });

  group('TitleBar', () {
    testWidgets('renders tab labels', (tester) async {
      await tester.pumpWidget(
        buildTestApp(_TitleBarHost(onClose: () {}), overrides: []),
      );
      await tester.pumpAndSettle();
      expect(find.text('Alpha'), findsOneWidget);
      expect(find.text('Beta'), findsOneWidget);
    });

    testWidgets('does not show Exit button on any platform', (tester) async {
      await tester.pumpWidget(
        buildTestApp(_TitleBarHost(onClose: () {}), overrides: []),
      );
      await tester.pumpAndSettle();
      expect(find.byIcon(Icons.power_settings_new), findsNothing);
    });

    // Close button is Windows-only; macOS uses native traffic lights.
    testWidgets(
      'close button calls onClose when tapped',
      skip: Platform.isMacOS,
      (tester) async {
        var called = false;
        await tester.pumpWidget(
          buildTestApp(
            _TitleBarHost(onClose: () => called = true),
            overrides: [],
          ),
        );
        await tester.pumpAndSettle();
        await tester.tap(find.byIcon(Icons.close));
        expect(called, isTrue);
      },
    );

    testWidgets(
      'close button turns red on hover and reverts on exit',
      skip: Platform.isMacOS,
      (tester) async {
        await tester.pumpWidget(
          buildTestApp(_TitleBarHost(onClose: () {}), overrides: []),
        );
        await tester.pumpAndSettle();

        final gesture = await tester.createGesture(
          kind: PointerDeviceKind.mouse,
        );
        await gesture.addPointer(location: Offset.zero);
        addTearDown(gesture.removePointer);

        await gesture.moveTo(tester.getCenter(find.byIcon(Icons.close)));
        await tester.pumpAndSettle();

        final containers = tester.widgetList<Container>(
          find.ancestor(
            of: find.byIcon(Icons.close),
            matching: find.byType(Container),
          ),
        );
        expect(
          containers.any((c) => c.color == const Color(0xFFE81123)),
          isTrue,
        );

        // Move away — triggers onExit, reverts hover state
        await gesture.moveTo(Offset.zero);
        await tester.pumpAndSettle();

        final containersAfter = tester.widgetList<Container>(
          find.ancestor(
            of: find.byIcon(Icons.close),
            matching: find.byType(Container),
          ),
        );
        expect(
          containersAfter.any((c) => c.color == const Color(0xFFE81123)),
          isFalse,
        );
      },
    );
  });
}

class _TitleBarHost extends StatefulWidget {
  const _TitleBarHost({required this.onClose});
  final VoidCallback onClose;

  @override
  State<_TitleBarHost> createState() => _TitleBarHostState();
}

class _TitleBarHostState extends State<_TitleBarHost>
    with SingleTickerProviderStateMixin {
  late final TabController _tc;

  @override
  void initState() {
    super.initState();
    _tc = TabController(length: 2, vsync: this);
  }

  @override
  void dispose() {
    _tc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return TitleBar(
      tabController: _tc,
      tabs: const ['Alpha', 'Beta'],
      onClose: widget.onClose,
    );
  }
}
