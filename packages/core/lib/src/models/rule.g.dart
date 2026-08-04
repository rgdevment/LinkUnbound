// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'rule.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

Rule _$RuleFromJson(Map<String, dynamic> json) => Rule(
  domain: json['domain'] as String,
  browserId: json['browserId'] as String,
  sourceApp: json['sourceApp'] as String?,
  private: json['private'] as bool? ?? false,
);

Map<String, dynamic> _$RuleToJson(Rule instance) => <String, dynamic>{
  'domain': instance.domain,
  'browserId': instance.browserId,
  'sourceApp': instance.sourceApp,
  'private': instance.private,
};
