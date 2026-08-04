import 'dart:io';

Set<String> get kLocalFileWebExtensions {
  if (Platform.isWindows) {
    return const {'.html', '.htm', '.xhtml', '.shtml', '.mhtml', '.pdf'};
  }
  return const {'.html', '.htm', '.xhtml'};
}

String? resolveLocalWebFile(String raw) {
  final path = _toFilesystemPath(raw);
  if (path == null) return null;

  if (!kLocalFileWebExtensions.contains(_extension(path))) return null;

  final file = File(path);
  if (!file.existsSync()) return null;

  try {
    return file.resolveSymbolicLinksSync();
  } on FileSystemException {
    return null;
  }
}

bool looksLikeLocalFile(String raw) {
  // Scheme comparison must be case-insensitive: `Uri` lowercases the scheme,
  // so a `FILE://` argument passes the inbound scheme check yet would skip
  // this guard — and with it the extension allowlist — if matched literally.
  if (Uri.tryParse(raw)?.scheme.toLowerCase() == 'file') return true;
  if (Platform.isWindows && _windowsAbsPath.hasMatch(raw)) return true;
  return false;
}

String redactPath(String path) {
  final parts = path
      .split(RegExp(r'[\\/]'))
      .where((p) => p.isNotEmpty)
      .toList();
  if (parts.isEmpty) return '<empty>';
  if (parts.length == 1) return '…/${parts.last}';
  return '…/${parts[parts.length - 2]}/${parts.last}';
}

String? _toFilesystemPath(String raw) {
  final uri = Uri.tryParse(raw);
  if (uri != null && uri.scheme.toLowerCase() == 'file') {
    // `file://host/share/x.html` becomes a UNC path, and merely probing it
    // makes Windows authenticate against `host`, leaking a NetNTLMv2 hash to
    // an attacker-chosen server before the picker is even shown.
    if (uri.host.isNotEmpty) return null;
    final String path;
    try {
      path = uri.toFilePath();
    } on UnsupportedError {
      return null;
    }
    if (path.startsWith(r'\\') || path.startsWith('//')) return null;
    return path;
  }

  if (Platform.isWindows && _windowsAbsPath.hasMatch(raw)) return raw;
  return null;
}

final RegExp _windowsAbsPath = RegExp(r'^[a-zA-Z]:[\\/]');

String _extension(String path) {
  final lastSep = path.lastIndexOf(RegExp(r'[\\/]'));
  final name = lastSep >= 0 ? path.substring(lastSep + 1) : path;
  final dot = name.lastIndexOf('.');
  if (dot <= 0) return '';
  return name.substring(dot).toLowerCase();
}
