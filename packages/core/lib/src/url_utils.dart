String stripEdgeProtocol(String raw) {
  const prefixes = [
    'microsoft-edge-https://',
    'microsoft-edge://',
    'microsoft-edge:',
  ];
  final lower = raw.toLowerCase();
  for (final prefix in prefixes) {
    if (lower.startsWith(prefix)) {
      final inner = raw.substring(prefix.length);
      if (prefix.contains('-https')) return 'https://$inner';
      return inner;
    }
  }
  return raw;
}

const _safeLinkHosts = {
  'statics.teams.cdn.office.net',
  'teams.public.onecdn.static.microsoft',
};

const _safeLinkPath = '/evergreen-assets/safelinks/';

bool _servesSafeLinks(Uri uri) {
  final host = uri.host.toLowerCase();
  if (host.endsWith('.safelinks.protection.outlook.com')) return true;
  if (_safeLinkHosts.contains(host)) return true;
  return uri.path.toLowerCase().startsWith(_safeLinkPath) &&
      (host.endsWith('.microsoft') || host.endsWith('.office.net'));
}

String unwrapSafeLink(String raw) {
  final uri = Uri.tryParse(raw);
  if (uri == null) return raw;

  if (!_servesSafeLinks(uri)) return raw;

  // `queryParameters` already percent-decodes; decoding again would resolve a
  // double-encoded `%2520` into a real character and change the destination.
  final inner = uri.queryParameters['url'];
  if (inner == null || inner.isEmpty) return raw;

  final innerUri = Uri.tryParse(inner);
  if (innerUri == null) return raw;
  if (innerUri.scheme != 'http' && innerUri.scheme != 'https') return raw;

  return inner;
}

/// Schemes LinkUnbound is willing to hand to a browser process.
const _launchableSchemes = {'http', 'https', 'file'};

/// Guards the boundary between an untrusted inbound URL and `Process.start`.
///
/// Browsers treat any argv entry starting with `-` (or `/` on Windows) as a
/// switch, so a crafted "URL" such as `--gpu-launcher=calc.exe` would make the
/// browser execute an arbitrary binary. Callers must reject anything this
/// returns false for before it reaches a launcher or the picker.
bool isLaunchableUrl(String raw) {
  if (raw.isEmpty) return false;
  if (raw.startsWith('-') || raw.startsWith('/') || raw.startsWith(r'\')) {
    return false;
  }
  final scheme = Uri.tryParse(raw)?.scheme.toLowerCase();
  return scheme != null && _launchableSchemes.contains(scheme);
}
