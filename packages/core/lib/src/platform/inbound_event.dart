import 'dart:convert';

sealed class InboundEvent {
  const InboundEvent();

  Map<String, dynamic> toJson();

  String encode() => jsonEncode(toJson());

  /// Decodes an event received over IPC. Every malformed shape must surface as
  /// a [FormatException] — callers only catch that type, and a stray
  /// [TypeError] would escape to the top-level error handler and be recorded
  /// as a crash for what is really just a bad message.
  static InboundEvent decode(String raw) {
    final Object? decoded;
    try {
      decoded = jsonDecode(raw);
    } on FormatException {
      rethrow;
    }
    if (decoded is! Map<String, dynamic>) {
      throw const FormatException('Inbound event is not a JSON object');
    }
    return fromJson(decoded);
  }

  static InboundEvent fromJson(Map<String, dynamic> json) =>
      switch (json['action']) {
        'open_url' when json['url'] is String => OpenUrlEvent(
          json['url'] as String,
          sourceApp: json['sourceApp'] as String?,
        ),
        'show_settings' => const ShowSettingsEvent(),
        _ => throw FormatException(
          'Unknown or malformed inbound event: ${json['action']}',
        ),
      };
}

final class OpenUrlEvent extends InboundEvent {
  const OpenUrlEvent(this.url, {this.sourceApp});

  final String url;

  /// App the link came from, lower-cased. Resolved by the process the shell
  /// launched — which is the only one that can see it — and carried across the
  /// IPC hop, since the resident instance has no way to work it out itself.
  final String? sourceApp;

  @override
  Map<String, dynamic> toJson() => {
    'action': 'open_url',
    'url': url,
    if (sourceApp != null) 'sourceApp': sourceApp,
  };
}

final class ShowSettingsEvent extends InboundEvent {
  const ShowSettingsEvent();

  @override
  Map<String, dynamic> toJson() => {'action': 'show_settings'};
}

abstract interface class InboundEventServer {
  Future<void> start();

  Stream<InboundEvent> get events;

  Future<void> stop();
}

abstract interface class InboundEventClient {
  Future<bool> send(InboundEvent event);
}
