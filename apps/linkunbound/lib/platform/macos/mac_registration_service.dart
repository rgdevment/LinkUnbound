import 'package:flutter/services.dart';
import 'package:linkunbound_core/linkunbound_core.dart';
import 'package:logging/logging.dart';

final _log = Logger('MacRegistrationService');

class MacRegistrationService implements RegistrationService {
  static const _channel = MethodChannel('linkunbound/registration');

  @override
  Future<void> register(String executablePath) async {
    await _channel.invokeMethod<void>('register');
  }

  /// No-op on macOS: Launch Services derives the handler from the bundle's
  /// `CFBundleURLTypes` and tracks the bundle wherever it moves, so there is
  /// no recorded path that can go stale. Becoming the *default* handler stays
  /// an explicit user action — re-asserting it on every launch would silently
  /// take the default back from another browser.
  @override
  Future<void> ensureRegistered(String executablePath) async {}

  /// Launch Services tracks the bundle by identity rather than by a recorded
  /// path, so there is no command that can go stale — the only thing worth
  /// reporting is whether we are the default handler.
  @override
  Future<HandlerDiagnostics> diagnose(String executablePath) async {
    return HandlerDiagnostics(
      isDefaultBrowser: await isDefault,
      commandMatchesExecutable: true,
      runningFromDevBuild: false,
      isPackaged: false,
    );
  }

  /// Windows-only concept: `microsoft-edge:` is not a scheme macOS apps use to
  /// open links.
  @override
  Future<void> setEdgeProtocolCapture(
    bool enabled,
    String executablePath,
  ) async {}

  @override
  Future<bool> get capturesEdgeProtocol async => false;

  @override
  Future<void> unregister() async {
    await _channel.invokeMethod<void>('unregister');
  }

  @override
  Future<bool> get isDefault async {
    try {
      final result = await _channel.invokeMethod<bool>('isDefault');
      return result ?? false;
    } on PlatformException catch (e, st) {
      _log.warning('isDefault check failed', e, st);
      return false;
    }
  }

  @override
  Future<Set<String>> get defaultAssociations async {
    try {
      final list = await _channel.invokeListMethod<String>(
        'defaultAssociations',
      );
      return (list ?? const <String>[]).toSet();
    } on PlatformException catch (e, st) {
      _log.warning('defaultAssociations failed', e, st);
      return const <String>{};
    }
  }
}
