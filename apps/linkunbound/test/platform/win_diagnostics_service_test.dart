import 'dart:convert';
import 'dart:io';

import 'package:archive/archive.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:linkunbound/platform/windows/win_diagnostics_service.dart';

void _stubRegistry(Directory staging) {
  File(
    '${staging.path}${Platform.pathSeparator}registry.txt',
  ).writeAsStringSync('');
}

void main() {
  group('parseWindowsVersion', () {
    test('returns Windows 11 for build 22000', () {
      const raw = 'Windows 10 Pro 10.0 (Build 22000)';
      expect(parseWindowsVersion(raw), contains('Windows 11'));
      expect(parseWindowsVersion(raw), isNot(contains('Windows 10')));
    });

    test('returns Windows 11 for build 26200', () {
      const raw = 'Windows 10 Pro 10.0 (Build 26200)';
      expect(parseWindowsVersion(raw), 'Windows 11 Pro 10.0 (Build 26200)');
    });

    test('keeps Windows 10 for build 21999', () {
      const raw = 'Windows 10 Pro 10.0 (Build 21999)';
      expect(parseWindowsVersion(raw), equals(raw));
    });

    test('returns raw string unchanged when no Build match', () {
      const raw = 'Windows 10 Pro 10.0';
      expect(parseWindowsVersion(raw), equals(raw));
    });

    test('does not modify string that already says Windows 11', () {
      const raw = 'Windows 11 Pro 10.0 (Build 22631)';
      expect(parseWindowsVersion(raw), equals(raw));
    });
  });

  group('exportDiagnostics', () {
    late Directory appDataDir;

    Future<String> export({String appVersion = '1.0.0'}) => exportDiagnostics(
      appDataDir: appDataDir,
      appVersion: appVersion,
      registryDumper: _stubRegistry,
      // A real revealer opens an Explorer window per test run.
      revealer: (_) async {},
    );

    setUp(() {
      appDataDir = Directory.systemTemp.createTempSync('lu_diag_test_');
    });

    tearDown(() {
      if (appDataDir.existsSync()) appDataDir.deleteSync(recursive: true);
    });

    List<ArchiveFile> zipFiles(String zipPath) {
      final bytes = File(zipPath).readAsBytesSync();
      return ZipDecoder().decodeBytes(bytes).files;
    }

    test('creates a zip file and returns its path', () async {
      final zipPath = await export(appVersion: '1.0.0-test');
      expect(File(zipPath).existsSync(), isTrue);
    });

    test('returned path is inside appDataDir', () async {
      final zipPath = await export();
      expect(zipPath, startsWith(appDataDir.path));
    });

    test('zip contains system_info.txt', () async {
      final zipPath = await export();
      final names = zipFiles(zipPath).map((f) => f.name).toList();
      expect(names, contains('system_info.txt'));
    });

    test('zip contains registry.txt', () async {
      final zipPath = await export();
      final names = zipFiles(zipPath).map((f) => f.name).toList();
      expect(names, contains('registry.txt'));
    });

    test('system_info.txt contains the app version', () async {
      final zipPath = await export(appVersion: '9.8.7-diag-test');
      final files = zipFiles(zipPath);
      final sysInfo = files.firstWhere((f) => f.name == 'system_info.txt');
      final content = utf8.decode(sysInfo.content as List<int>);
      expect(content, contains('9.8.7-diag-test'));
    });

    test('system_info.txt includes appDataDir path', () async {
      final zipPath = await export();
      final files = zipFiles(zipPath);
      final sysInfo = files.firstWhere((f) => f.name == 'system_info.txt');
      final content = utf8.decode(sysInfo.content as List<int>);
      expect(content, contains(appDataDir.path));
    });

    test('includes navigate.log when file exists', () async {
      File(
        '${appDataDir.path}${Platform.pathSeparator}navigate.log',
      ).writeAsStringSync('line1\nline2\nline3');

      final zipPath = await export();
      final names = zipFiles(zipPath).map((f) => f.name).toList();
      expect(names, contains('navigate.log'));
    });

    test('navigate.log content is preserved when small', () async {
      File(
        '${appDataDir.path}${Platform.pathSeparator}navigate.log',
      ).writeAsStringSync('alpha\nbeta\ngamma');

      final zipPath = await export();
      final files = zipFiles(zipPath);
      final logFile = files.firstWhere((f) => f.name == 'navigate.log');
      final content = utf8.decode(logFile.content as List<int>);
      expect(content, contains('alpha'));
      expect(content, contains('gamma'));
    });

    test('navigate.log is truncated to last 200 lines when large', () async {
      final lines = List.generate(350, (i) => 'entry $i');
      File(
        '${appDataDir.path}${Platform.pathSeparator}navigate.log',
      ).writeAsStringSync(lines.join('\n'));

      final zipPath = await export();
      final files = zipFiles(zipPath);
      final logFile = files.firstWhere((f) => f.name == 'navigate.log');
      final content = utf8.decode(logFile.content as List<int>);
      final resultLines = content
          .split('\n')
          .where((l) => l.isNotEmpty)
          .toList();
      expect(resultLines.length, 200);
      expect(resultLines.first, 'entry 150');
      expect(resultLines.last, 'entry 349');
    });

    test('navigate.log missing is silently skipped', () async {
      final zipPath = await export();
      final names = zipFiles(zipPath).map((f) => f.name).toList();
      expect(names, isNot(contains('navigate.log')));
    });

    test('data files listed in system_info.txt', () async {
      File(
        '${appDataDir.path}${Platform.pathSeparator}browsers.json',
      ).writeAsStringSync('[]');

      final zipPath = await export();
      final files = zipFiles(zipPath);
      final sysInfo = files.firstWhere((f) => f.name == 'system_info.txt');
      final content = utf8.decode(sysInfo.content as List<int>);
      expect(content, contains('browsers.json'));
    });

    test('subdirectory in appDataDir is listed with trailing slash', () async {
      Directory(
        '${appDataDir.path}${Platform.pathSeparator}icons',
      ).createSync();

      final zipPath = await export();
      final files = zipFiles(zipPath);
      final sysInfo = files.firstWhere((f) => f.name == 'system_info.txt');
      final content = utf8.decode(sysInfo.content as List<int>);
      expect(content, contains('icons/'));
    });

    test('system_info.txt contains OS version line', () async {
      final zipPath = await export();
      final files = zipFiles(zipPath);
      final sysInfo = files.firstWhere((f) => f.name == 'system_info.txt');
      final content = utf8.decode(sysInfo.content as List<int>);
      expect(content, contains('OS:'));
    });
  });
}
