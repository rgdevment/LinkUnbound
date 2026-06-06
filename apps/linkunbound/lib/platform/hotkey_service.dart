import 'dart:io' show Platform;

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:hotkey_manager/hotkey_manager.dart';
import 'package:logging/logging.dart';

final _log = Logger('HotkeyService');

/// Manages a single global hotkey.  Call [register] to activate it; the
/// [onTriggered] callback fires every time the shortcut is pressed.
/// Call [dispose] to unregister before the app exits.
final class HotkeyService {
  HotKey? _activeKey;
  VoidCallback? _onTriggered;

  bool get isRegistered => _activeKey != null;

  void setCallback(VoidCallback callback) {
    _onTriggered = callback;
  }

  /// Parses [serialized] (e.g. "meta+alt+space") and registers the hotkey.
  /// No-op if [serialized] is null/empty.  Unregisters any previous hotkey first.
  Future<void> register(String? serialized) async {
    await unregister();
    if (serialized == null || serialized.trim().isEmpty) return;
    final key = parse(serialized);
    if (key == null) {
      _log.warning('Could not parse hotkey: $serialized');
      return;
    }
    try {
      await hotKeyManager.register(
        key,
        keyDownHandler: (_) => _onTriggered?.call(),
      );
      _activeKey = key;
      _log.info('Global hotkey registered: $serialized');
    } on Exception catch (e) {
      _log.warning('Failed to register hotkey "$serialized": $e');
    }
  }

  Future<void> unregister() async {
    final key = _activeKey;
    if (key == null) return;
    try {
      await hotKeyManager.unregister(key);
    } on Exception catch (e) {
      _log.warning('Failed to unregister hotkey: $e');
    }
    _activeKey = null;
  }

  Future<void> dispose() async {
    await unregister();
    _onTriggered = null;
  }

  /// Serialises a [HotKey] to the stable `modifiers+key` string stored
  /// in the settings file.
  static String serialize(HotKey hotKey) {
    final parts = <String>[
      if (hotKey.modifiers != null)
        for (final m in hotKey.modifiers!)
          _modifierLabel(m),
      _keyLabel(hotKey.key),
    ];
    return parts.join('+');
  }

  @visibleForTesting
  static HotKey? parse(String raw) {
    final parts = raw.toLowerCase().split('+');
    if (parts.isEmpty) return null;

    final modifiers = <HotKeyModifier>[];
    String? keyPart;
    for (final p in parts) {
      switch (p.trim()) {
        case 'meta':
        case 'cmd':
        case 'win':
          modifiers.add(HotKeyModifier.meta);
        case 'alt':
        case 'option':
          modifiers.add(HotKeyModifier.alt);
        case 'ctrl':
        case 'control':
          modifiers.add(HotKeyModifier.control);
        case 'shift':
          modifiers.add(HotKeyModifier.shift);
        default:
          keyPart = p.trim();
      }
    }
    if (keyPart == null) return null;
    final key = _keysByLabel[keyPart];
    if (key == null) return null;
    return HotKey(
      key: key,
      modifiers: modifiers.isEmpty ? null : modifiers,
      scope: HotKeyScope.system,
    );
  }

  static String _modifierLabel(HotKeyModifier m) {
    if (m == HotKeyModifier.meta) return 'meta';
    if (m == HotKeyModifier.alt) return 'alt';
    if (m == HotKeyModifier.control) return 'ctrl';
    if (m == HotKeyModifier.shift) return 'shift';
    return m.name.toLowerCase();
  }

  static String _keyLabel(KeyboardKey key) {
    for (final entry in _keysByLabel.entries) {
      if (entry.value == key) return entry.key;
    }
    return 'unknown';
  }

  // Only keys reachable from the presets/serialized format; parse() rejects
  // anything outside this map.
  static const Map<String, PhysicalKeyboardKey> _keysByLabel = {
    'space': PhysicalKeyboardKey.space,
    'a': PhysicalKeyboardKey.keyA,
    'b': PhysicalKeyboardKey.keyB,
    'c': PhysicalKeyboardKey.keyC,
    'd': PhysicalKeyboardKey.keyD,
    'e': PhysicalKeyboardKey.keyE,
    'f': PhysicalKeyboardKey.keyF,
    'g': PhysicalKeyboardKey.keyG,
    'h': PhysicalKeyboardKey.keyH,
    'i': PhysicalKeyboardKey.keyI,
    'j': PhysicalKeyboardKey.keyJ,
    'k': PhysicalKeyboardKey.keyK,
    'l': PhysicalKeyboardKey.keyL,
    'm': PhysicalKeyboardKey.keyM,
    'n': PhysicalKeyboardKey.keyN,
    'o': PhysicalKeyboardKey.keyO,
    'p': PhysicalKeyboardKey.keyP,
    'q': PhysicalKeyboardKey.keyQ,
    'r': PhysicalKeyboardKey.keyR,
    's': PhysicalKeyboardKey.keyS,
    't': PhysicalKeyboardKey.keyT,
    'u': PhysicalKeyboardKey.keyU,
    'v': PhysicalKeyboardKey.keyV,
    'w': PhysicalKeyboardKey.keyW,
    'x': PhysicalKeyboardKey.keyX,
    'y': PhysicalKeyboardKey.keyY,
    'z': PhysicalKeyboardKey.keyZ,
    'f1': PhysicalKeyboardKey.f1,
    'f2': PhysicalKeyboardKey.f2,
    'f3': PhysicalKeyboardKey.f3,
    'f4': PhysicalKeyboardKey.f4,
    'f5': PhysicalKeyboardKey.f5,
    'f6': PhysicalKeyboardKey.f6,
    'f7': PhysicalKeyboardKey.f7,
    'f8': PhysicalKeyboardKey.f8,
    'f9': PhysicalKeyboardKey.f9,
    'f10': PhysicalKeyboardKey.f10,
    'f11': PhysicalKeyboardKey.f11,
    'f12': PhysicalKeyboardKey.f12,
  };
}

/// Preset hotkey options shown in the settings dropdown.
final class HotkeyPreset {
  const HotkeyPreset({required this.label, required this.serialized});
  final String label;
  final String serialized;

  /// Per-OS lists: `meta` is Cmd on macOS but the Win key on Windows, so the
  /// shortcuts themselves differ, not just their labels.
  static List<HotkeyPreset> get defaults =>
      Platform.isMacOS ? _macDefaults : _windowsDefaults;

  static const List<HotkeyPreset> _macDefaults = [
    HotkeyPreset(label: '⌘ ⇧ L', serialized: 'meta+shift+l'),
    HotkeyPreset(label: '⌘ ⇧ B', serialized: 'meta+shift+b'),
    HotkeyPreset(label: '⌃ ⌥ L', serialized: 'ctrl+alt+l'),
    HotkeyPreset(label: '⌘ ⌥ Space', serialized: 'meta+alt+space'),
  ];

  static const List<HotkeyPreset> _windowsDefaults = [
    HotkeyPreset(label: 'Ctrl+Shift+L', serialized: 'ctrl+shift+l'),
    HotkeyPreset(label: 'Ctrl+Shift+B', serialized: 'ctrl+shift+b'),
    HotkeyPreset(label: 'Ctrl+Alt+L', serialized: 'ctrl+alt+l'),
    HotkeyPreset(label: 'Ctrl+Alt+Space', serialized: 'ctrl+alt+space'),
  ];
}
