import 'dart:convert';
import 'dart:io';

final class UpdateInfo {
  const UpdateInfo({required this.latestVersion, required this.releaseUrl});

  final String latestVersion;
  final String releaseUrl;
}

final class UpdateService {
  const UpdateService({required this.owner, required this.repo});

  final String owner;
  final String repo;

  Future<UpdateInfo?> checkForUpdate(String currentVersion) async {
    final client = HttpClient();
    try {
      client.connectionTimeout = const Duration(seconds: 5);

      final request = await client.getUrl(
        Uri.parse('https://api.github.com/repos/$owner/$repo/releases/latest'),
      );
      request.headers.set('Accept', 'application/vnd.github.v3+json');
      request.headers.set('User-Agent', 'LinkUnbound/$currentVersion');

      final response = await request.close();
      if (response.statusCode != 200) return null;

      final body = await response.transform(utf8.decoder).join();

      final decoded = jsonDecode(body);
      if (decoded is! Map<String, dynamic>) return null;
      final tagName = decoded['tag_name'] as String?;
      final htmlUrl = decoded['html_url'] as String?;
      if (tagName == null || htmlUrl == null) return null;
      // The release URL ends up in a shell "open" call, so it must not be
      // taken on trust from the response body: a compromised or intercepted
      // endpoint could otherwise point it at an executable.
      if (!isTrustedReleaseUrl(htmlUrl)) return null;

      final version = tagName.startsWith('v') ? tagName.substring(1) : tagName;

      if (!_isNewer(version, currentVersion)) return null;

      return UpdateInfo(latestVersion: version, releaseUrl: htmlUrl);
    } on Exception {
      return null;
    } finally {
      // Previously leaked on every error path.
      client.close();
    }
  }

  /// True only for HTTPS URLs served by github.com itself.
  static bool isTrustedReleaseUrl(String raw) {
    final uri = Uri.tryParse(raw);
    if (uri == null || uri.scheme != 'https') return false;
    final host = uri.host.toLowerCase();
    return host == 'github.com' || host.endsWith('.github.com');
  }

  static bool _isNewer(String latest, String current) {
    final partsL = latest.split('.').map(int.tryParse).toList();
    final partsC = current.split('.').map(int.tryParse).toList();
    for (var i = 0; i < 3; i++) {
      final l = i < partsL.length ? (partsL[i] ?? 0) : 0;
      final c = i < partsC.length ? (partsC[i] ?? 0) : 0;
      if (l > c) return true;
      if (l < c) return false;
    }
    return false;
  }
}
