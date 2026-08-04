// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'browser.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

Browser _$BrowserFromJson(Map<String, dynamic> json) => Browser(
  id: json['id'] as String,
  name: json['name'] as String,
  executablePath: json['executablePath'] as String,
  iconPath: json['iconPath'] as String,
  extraArgs:
      (json['extraArgs'] as List<dynamic>?)?.map((e) => e as String).toList() ??
      const [],
  isCustom: json['isCustom'] as bool? ?? false,
  privateArgs: (json['privateArgs'] as List<dynamic>?)
      ?.map((e) => e as String)
      .toList(),
);

Map<String, dynamic> _$BrowserToJson(Browser instance) => <String, dynamic>{
  'id': instance.id,
  'name': instance.name,
  'executablePath': instance.executablePath,
  'iconPath': instance.iconPath,
  'extraArgs': instance.extraArgs,
  'isCustom': instance.isCustom,
  'privateArgs': instance.privateArgs,
};
