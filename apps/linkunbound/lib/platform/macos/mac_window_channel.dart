import 'package:flutter/services.dart';
import 'package:logging/logging.dart';

final _log = Logger('MacWindowChannel');

class MacWindowChannel {
  static const _channel = MethodChannel('linkunbound/window');

  Future<void> setPickerMode() => _invoke('setPickerMode');

  Future<void> setSettingsMode() => _invoke('setSettingsMode');

  Future<void> activate() => _invoke('activate');

  Future<void> setRegular() => _invoke('setRegular');

  Future<void> setAccessory() => _invoke('setAccessory');

  Future<void> _invoke(String method) async {
    try {
      await _channel.invokeMethod<void>(method);
    } on PlatformException catch (e, st) {
      _log.warning('Window channel "$method" failed', e, st);
    }
  }
}
