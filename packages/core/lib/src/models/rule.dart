import 'package:json_annotation/json_annotation.dart';

part 'rule.g.dart';

/// Matches any domain. Used by rules that key off the originating app alone,
/// e.g. "everything opened from Slack goes to Brave".
const kAnyDomain = '*';

@JsonSerializable()
final class Rule {
  const Rule({
    required this.domain,
    required this.browserId,
    this.sourceApp,
    this.private = false,
  });

  factory Rule.fromJson(Map<String, dynamic> json) => _$RuleFromJson(json);

  final String domain;
  final String browserId;

  /// Identifier of the app the link came from, lower-cased — the executable
  /// name on Windows ("slack") and the bundle id on macOS. Null means the rule
  /// applies whatever the origin is.
  final String? sourceApp;

  /// Whether matching links open in a private window.
  final bool private;

  /// True when this rule only constrains the originating app.
  bool get matchesAnyDomain => domain == kAnyDomain;

  Map<String, dynamic> toJson() => _$RuleToJson(this);

  /// [sourceApp] is nullable and meaningful when null, so clearing it needs an
  /// explicit flag rather than the usual `?? this.x` idiom.
  Rule copyWith({
    String? domain,
    String? browserId,
    String? sourceApp,
    bool clearSourceApp = false,
    bool? private,
  }) => Rule(
    domain: domain ?? this.domain,
    browserId: browserId ?? this.browserId,
    sourceApp: clearSourceApp ? null : (sourceApp ?? this.sourceApp),
    private: private ?? this.private,
  );
}
