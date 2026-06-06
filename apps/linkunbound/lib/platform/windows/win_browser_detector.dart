import 'package:flutter/foundation.dart';
import 'package:logging/logging.dart';
import 'package:linkunbound_core/linkunbound_core.dart';
import 'package:win32_registry/win32_registry.dart';

final _log = Logger('WinBrowserDetector');

/// Parses the executable path from a registry shell/open/command value.
/// Handles quoted paths and paths where an intermediate directory name
/// contains '.exe' (e.g. C:\some.exe.dir\chrome.exe --args).
@visibleForTesting
String extractExePath(String rawCommand) {
  final trimmed = rawCommand.trim();
  if (trimmed.isEmpty) return '';

  if (trimmed.startsWith('"')) {
    final end = trimmed.indexOf('"', 1);
    if (end < 0) return trimmed.substring(1);
    return trimmed.substring(1, end);
  }

  // Find the last '.exe' immediately followed by a non-path character so that
  // intermediate directory components like "foo.exe.d\" are skipped.
  final lower = trimmed.toLowerCase();
  var searchFrom = lower.length;
  while (true) {
    final idx = lower.lastIndexOf('.exe', searchFrom - 1);
    if (idx < 0) break;
    final after = idx + 4;
    if (after == lower.length || lower[after] == ' ' || lower[after] == '%') {
      return trimmed.substring(0, after);
    }
    searchFrom = idx;
  }

  final spaceIndex = trimmed.indexOf(' ');
  if (spaceIndex < 0) return trimmed;
  return trimmed.substring(0, spaceIndex);
}

final class WinBrowserDetector implements BrowserDetector {
  @override
  Future<List<Browser>> detect() async {
    final browsers = <String, Browser>{};

    for (final hive in [RegistryHive.localMachine, RegistryHive.currentUser]) {
      _scanHive(hive, browsers);
    }

    return browsers.values.toList();
  }

  static const _blockedBrowsers = {
    'iexplore',
    'iexplore.exe',
    'internet explorer',
  };

  void _scanHive(RegistryHive hive, Map<String, Browser> browsers) {
    final RegistryKey root;
    try {
      root = Registry.openPath(
        hive,
        path: r'Software\Clients\StartMenuInternet',
      );
    } on Exception {
      return;
    }

    try {
      for (final name in root.subkeyNames) {
        if (name == 'LinkUnbound') continue;
        if (browsers.containsKey(name.toLowerCase())) continue;
        if (_blockedBrowsers.contains(name.toLowerCase())) continue;

        try {
          final browser = _readBrowser(hive, name);
          if (browser != null) {
            browsers[name.toLowerCase()] = browser;
          }
        } on Exception catch (e) {
          _log.warning('Failed to read browser $name: $e');
        }
      }
    } finally {
      root.close();
    }
  }

  Browser? _readBrowser(RegistryHive hive, String name) {
    final basePath = 'Software\\Clients\\StartMenuInternet\\$name';
    final key = Registry.openPath(hive, path: basePath);

    try {
      final displayName =
          key.getValueAsString('') ?? key.getValueAsString('(Default)') ?? name;

      final commandKey = Registry.openPath(
        hive,
        path: '$basePath\\shell\\open\\command',
      );
      final rawCommand = commandKey.getValueAsString('') ?? '';
      commandKey.close();

      final executablePath = _extractExePath(rawCommand);
      if (executablePath.isEmpty) return null;

      final id = _generateId(name);

      String iconPath = '';
      try {
        final iconKey = Registry.openPath(hive, path: '$basePath\\DefaultIcon');
        iconPath = _extractIconPath(iconKey.getValueAsString('') ?? '');
        iconKey.close();
      } on Exception {
        iconPath = executablePath;
      }

      return Browser(
        id: id,
        name: displayName,
        executablePath: executablePath,
        iconPath: iconPath,
      );
    } finally {
      key.close();
    }
  }

  String _extractExePath(String rawCommand) => extractExePath(rawCommand);

  String _extractIconPath(String raw) {
    final trimmed = raw.trim();
    if (trimmed.isEmpty) return '';

    if (trimmed.startsWith('"')) {
      final end = trimmed.indexOf('"', 1);
      if (end < 0) return trimmed.substring(1);
      return trimmed.substring(1, end);
    }

    final commaIndex = trimmed.lastIndexOf(',');
    if (commaIndex > 0) return trimmed.substring(0, commaIndex).trim();
    return trimmed;
  }

  String _generateId(String registryName) => registryName
      .toLowerCase()
      .replaceAll(RegExp(r'[^a-z0-9]+'), '-')
      .replaceAll(RegExp(r'^-|-$'), '');
}
