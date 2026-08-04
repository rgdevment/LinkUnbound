import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:linkunbound_core/linkunbound_core.dart';
import 'package:test/test.dart';

// --- Minimal HTTP stub chain for intercepting UpdateService's HttpClient ---

final class _StubHeaders implements HttpHeaders {
  @override
  void set(String name, Object value, {bool preserveHeaderCase = false}) {}

  @override
  dynamic noSuchMethod(Invocation invocation) => null;
}

final class _StubResponse extends Stream<List<int>>
    implements HttpClientResponse {
  _StubResponse(this._status, this._body);
  final int _status;
  final String _body;

  @override
  int get statusCode => _status;

  @override
  StreamSubscription<List<int>> listen(
    void Function(List<int> event)? onData, {
    Function? onError,
    void Function()? onDone,
    bool? cancelOnError,
  }) => Stream.value(utf8.encode(_body)).listen(
    onData,
    onError: onError,
    onDone: onDone,
    cancelOnError: cancelOnError,
  );

  @override
  dynamic noSuchMethod(Invocation invocation) => null;
}

final class _StubRequest implements HttpClientRequest {
  _StubRequest(this._status, this._body);
  final int _status;
  final String _body;

  @override
  HttpHeaders get headers => _StubHeaders();

  @override
  Future<HttpClientResponse> close() async => _StubResponse(_status, _body);

  @override
  dynamic noSuchMethod(Invocation invocation) => null;
}

final class _StubHttpClient implements HttpClient {
  _StubHttpClient(this._status, this._body);
  final int _status;
  final String _body;

  @override
  Duration? connectionTimeout;

  @override
  Future<HttpClientRequest> getUrl(Uri url) async =>
      _StubRequest(_status, _body);

  @override
  void close({bool force = false}) {}

  @override
  dynamic noSuchMethod(Invocation invocation) => null;
}

Future<UpdateInfo?> _check({
  required int status,
  required Map<String, dynamic> body,
  required String current,
}) => HttpOverrides.runZoned(
  () => const UpdateService(owner: 'o', repo: 'r').checkForUpdate(current),
  createHttpClient: (_) => _StubHttpClient(status, jsonEncode(body)),
);

// ---------------------------------------------------------------------------

void main() {
  group('UpdateInfo', () {
    test('stores latestVersion and releaseUrl', () {
      const info = UpdateInfo(
        latestVersion: '2.1.0',
        releaseUrl: 'https://github.com/owner/repo/releases/tag/v2.1.0',
      );
      expect(info.latestVersion, '2.1.0');
      expect(
        info.releaseUrl,
        'https://github.com/owner/repo/releases/tag/v2.1.0',
      );
    });

    test('fields are accessible after construction', () {
      const info = UpdateInfo(
        latestVersion: '1.0.0',
        releaseUrl: 'https://github.com/o/r/releases/tag/x',
      );
      expect(info.latestVersion, isNotEmpty);
      expect(info.releaseUrl, isNotEmpty);
    });
  });

  group('UpdateService', () {
    test('stores owner and repo', () {
      const service = UpdateService(owner: 'rgdevment', repo: 'LinkUnbound');
      expect(service.owner, 'rgdevment');
      expect(service.repo, 'LinkUnbound');
    });

    test(
      'checkForUpdate returns null on network failure',
      () async {
        // Unresolvable host triggers the on-Exception path → null
        const service = UpdateService(
          owner: 'localhost-unreachable-host-xyz',
          repo: 'repo',
        );
        final result = await service.checkForUpdate('1.0.0');
        expect(result, isNull);
      },
      timeout: const Timeout(Duration(seconds: 10)),
    );
  });

  group('UpdateService HTTP paths', () {
    test('returns UpdateInfo when newer major version available', () async {
      final result = await _check(
        status: 200,
        body: {
          'tag_name': 'v2.0.0',
          'html_url': 'https://github.com/o/r/releases/tag/v2.0.0',
        },
        current: '1.0.0',
      );
      expect(result, isNotNull);
      expect(result!.latestVersion, '2.0.0');
      expect(result.releaseUrl, 'https://github.com/o/r/releases/tag/v2.0.0');
    });

    test('returns null when html_url is not a github.com URL', () async {
      // The release URL is handed to the shell, so an untrusted host must not
      // survive the parse even if the version looks newer.
      final result = await _check(
        status: 200,
        body: {
          'tag_name': 'v2.0.0',
          'html_url': 'https://evil.example.com/payload.exe',
        },
        current: '1.0.0',
      );
      expect(result, isNull);
    });

    test('returns null when html_url uses a non-https scheme', () async {
      final result = await _check(
        status: 200,
        body: {
          'tag_name': 'v2.0.0',
          'html_url': 'file://attacker/share/payload.exe',
        },
        current: '1.0.0',
      );
      expect(result, isNull);
    });

    test('strips v prefix from tag_name', () async {
      final result = await _check(
        status: 200,
        body: {
          'tag_name': 'v1.5.0',
          'html_url': 'https://github.com/o/r/releases/tag/x',
        },
        current: '1.0.0',
      );
      expect(result!.latestVersion, '1.5.0');
    });

    test('accepts tag_name without v prefix', () async {
      final result = await _check(
        status: 200,
        body: {
          'tag_name': '1.5.0',
          'html_url': 'https://github.com/o/r/releases/tag/x',
        },
        current: '1.0.0',
      );
      expect(result!.latestVersion, '1.5.0');
    });

    test('returns null when version is equal to current', () async {
      final result = await _check(
        status: 200,
        body: {
          'tag_name': 'v1.0.0',
          'html_url': 'https://github.com/o/r/releases/tag/x',
        },
        current: '1.0.0',
      );
      expect(result, isNull);
    });

    test('returns null when latest is older than current', () async {
      final result = await _check(
        status: 200,
        body: {
          'tag_name': 'v0.9.9',
          'html_url': 'https://github.com/o/r/releases/tag/x',
        },
        current: '1.0.0',
      );
      expect(result, isNull);
    });

    test('returns null on non-200 status', () async {
      final result = await _check(status: 404, body: {}, current: '1.0.0');
      expect(result, isNull);
    });

    test('returns null when tag_name is null', () async {
      final result = await _check(
        status: 200,
        body: {
          'tag_name': null,
          'html_url': 'https://github.com/o/r/releases/tag/x',
        },
        current: '1.0.0',
      );
      expect(result, isNull);
    });

    test('returns null when html_url is null', () async {
      final result = await _check(
        status: 200,
        body: {'tag_name': 'v2.0.0', 'html_url': null},
        current: '1.0.0',
      );
      expect(result, isNull);
    });

    test('minor version bump triggers update', () async {
      final result = await _check(
        status: 200,
        body: {
          'tag_name': 'v1.1.0',
          'html_url': 'https://github.com/o/r/releases/tag/x',
        },
        current: '1.0.9',
      );
      expect(result, isNotNull);
    });

    test('patch version bump triggers update', () async {
      final result = await _check(
        status: 200,
        body: {
          'tag_name': 'v1.0.1',
          'html_url': 'https://github.com/o/r/releases/tag/x',
        },
        current: '1.0.0',
      );
      expect(result, isNotNull);
    });

    test('major rollback returns null', () async {
      final result = await _check(
        status: 200,
        body: {
          'tag_name': 'v2.0.0',
          'html_url': 'https://github.com/o/r/releases/tag/x',
        },
        current: '3.0.0',
      );
      expect(result, isNull);
    });

    test('minor rollback returns null', () async {
      final result = await _check(
        status: 200,
        body: {
          'tag_name': 'v1.0.0',
          'html_url': 'https://github.com/o/r/releases/tag/x',
        },
        current: '1.1.0',
      );
      expect(result, isNull);
    });
  });
}
