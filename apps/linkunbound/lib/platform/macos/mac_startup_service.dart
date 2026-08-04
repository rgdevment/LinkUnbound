import 'package:flutter/services.dart';
import 'package:linkunbound_core/linkunbound_core.dart';
import 'package:logging/logging.dart';

final _log = Logger('MacStartupService');

class MacStartupService implements StartupService {
  static const _channel = MethodChannel('linkunbound/startup');

  @override
  Future<void> enable(String executablePath) async {
    await _channel.invokeMethod<void>('enable');
  }

  @override
  Future<void> disable() async {
    await _channel.invokeMethod<void>('disable');
  }

  @override
  Future<bool> get isEnabled async {
    try {
      final result = await _channel.invokeMethod<bool>('isEnabled');
      return result ?? false;
    } on PlatformException catch (e, st) {
      _log.warning('isEnabled check failed', e, st);
      return false;
    }
  }

  /// Returns true when the process was launched by the macOS login item
  /// mechanism (parent PID == launchd). Used to suppress the Settings window
  /// on login-triggered starts.
  Future<bool> get isLoginItemLaunch async {
    try {
      final result = await _channel.invokeMethod<bool>('isLoginItemLaunch');
      return result ?? false;
      // Catches Object, not PlatformException: this runs before bootstrap, and
      // if the channel is not registered yet the MissingPluginException would
      // escape MacOsBindings.create() and kill the launch outright — no window,
      // no tray, no link handling.
    } on Object catch (e, st) {
      _log.warning('isLoginItemLaunch check failed', e, st);
      return false;
    }
  }
}
