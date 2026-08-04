import 'package:json_annotation/json_annotation.dart';

import '../private_mode.dart';

part 'browser.g.dart';

@JsonSerializable()
final class Browser {
  const Browser({
    required this.id,
    required this.name,
    required this.executablePath,
    required this.iconPath,
    this.extraArgs = const [],
    this.isCustom = false,
    this.privateArgs,
  });

  factory Browser.fromJson(Map<String, dynamic> json) =>
      _$BrowserFromJson(json);

  final String id;
  final String name;
  final String executablePath;
  final String iconPath;
  final List<String> extraArgs;
  final bool isCustom;

  /// Overrides the private-window switch derived from the executable name.
  /// Null means "derive it"; an empty list means "this browser has none",
  /// which is how a custom browser opts out.
  final List<String>? privateArgs;

  /// Arguments that open this browser in a private window, empty when it does
  /// not support one.
  List<String> get resolvedPrivateArgs =>
      privateArgs ?? privateModeArgs(executablePath);

  bool get canOpenPrivately => resolvedPrivateArgs.isNotEmpty;

  Map<String, dynamic> toJson() => _$BrowserToJson(this);

  Browser copyWith({
    String? id,
    String? name,
    String? executablePath,
    String? iconPath,
    List<String>? extraArgs,
    bool? isCustom,
    List<String>? privateArgs,
    bool clearPrivateArgs = false,
  }) => Browser(
    id: id ?? this.id,
    name: name ?? this.name,
    executablePath: executablePath ?? this.executablePath,
    iconPath: iconPath ?? this.iconPath,
    extraArgs: extraArgs ?? this.extraArgs,
    isCustom: isCustom ?? this.isCustom,
    privateArgs: clearPrivateArgs ? null : (privateArgs ?? this.privateArgs),
  );
}
