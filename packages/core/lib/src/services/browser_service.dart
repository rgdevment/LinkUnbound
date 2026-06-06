import 'dart:convert';
import 'dart:io';

import 'package:logging/logging.dart';

import '../models/browser.dart';
import '../models/browser_config.dart';
import '../platform/browser_detector.dart';

final class BrowserService {
  BrowserService({required this.configFile, required this.browserDetector});

  final File configFile;
  final BrowserDetector browserDetector;
  final _log = Logger('BrowserService');

  List<Browser> _browsers = [];

  // Rebuilt only when _browsers is replaced, avoiding per-access allocation.
  List<Browser> _browsersView = const [];

  List<Browser> get browsers => _browsersView;

  void _setBrowsers(List<Browser> list) {
    _browsers = list;
    _browsersView = List.unmodifiable(list);
  }

  Future<void> load() async {
    if (!configFile.existsSync()) {
      _setBrowsers([]);
      return;
    }
    final content = await configFile.readAsString();
    final config = BrowserConfig.fromJson(
      jsonDecode(content) as Map<String, dynamic>,
    );
    _setBrowsers(config.browsers);
  }

  Future<void> save() async {
    await configFile.parent.create(recursive: true);
    final config = BrowserConfig(browsers: _browsers);
    const encoder = JsonEncoder.withIndent('  ');
    await configFile.writeAsString(encoder.convert(config.toJson()));
  }

  Future<({int added, int removed})> scanAndMerge() async {
    final detected = await browserDetector.detect();
    final detectedById = {for (final d in detected) d.id: d};

    final removedCount = _browsers
        .where((b) => !b.isCustom && !detectedById.containsKey(b.id))
        .length;

    // Keep existing browsers that are still installed (or custom), preserving order
    final kept = [
      for (final b in _browsers)
        if (b.isCustom)
          b
        else if (detectedById[b.id] case final d?)
          b.copyWith(executablePath: d.executablePath, iconPath: d.iconPath),
    ];

    final existingIds = {for (final b in _browsers) b.id};
    final newBrowsers = detected
        .where((d) => !existingIds.contains(d.id))
        .toList();

    _setBrowsers([...kept, ...newBrowsers]);
    if (newBrowsers.isNotEmpty || removedCount > 0) {
      _log.info(
        'Browsers updated: ${newBrowsers.length} added, '
        '$removedCount removed, ${kept.length} kept',
      );
    }
    return (added: newBrowsers.length, removed: removedCount);
  }

  void addBrowser(Browser browser) {
    _setBrowsers([..._browsers, browser]);
  }

  void removeBrowser(String id) {
    _setBrowsers(_browsers.where((b) => b.id != id).toList());
  }

  void updateBrowser(String id, Browser browser) {
    _setBrowsers([
      for (final b in _browsers)
        if (b.id == id) browser else b,
    ]);
  }

  void reorder(int oldIndex, int newIndex) {
    final list = [..._browsers];
    final item = list.removeAt(oldIndex);
    list.insert(newIndex.clamp(0, list.length), item);
    _setBrowsers(list);
  }

  Future<void> reset() async {
    _setBrowsers([]);
    if (configFile.existsSync()) {
      await configFile.delete();
    }
    _log.warning('Browser config reset');
  }
}
