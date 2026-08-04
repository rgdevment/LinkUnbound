import 'package:flutter/services.dart';
import 'package:logging/logging.dart';

final _log = Logger('MacSourceApp');

const _channel = MethodChannel('linkunbound/source_app');

/// Best-effort identification of the app that opened the link.
/// macOS exposes no originator for open events, so the frontmost
/// application is used as an approximation.
Future<({String id, String name})?> frontmostApp() async {
  try {
    final raw = await _channel.invokeMapMethod<String, String>('frontmostApp');
    final id = raw?['id'];
    final name = raw?['name'];
    if (id == null || name == null) return null;
    return (id: id, name: name);
    // Catches Object, not PlatformException: this runs on the launch path, and
    // an unregistered channel raises MissingPluginException, which would escape
    // and kill the launch outright instead of just losing the source app.
  } on Object catch (e, st) {
    _log.fine('frontmostApp lookup failed', e, st);
    return null;
  }
}
