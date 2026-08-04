import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';
import 'package:flutter/foundation.dart';

/// Returned by `GetCurrentPackageFullName` when the process has no package
/// identity, i.e. it is not running inside an MSIX container.
const _appmodelErrorNoPackage = 15700;

/// Path fragment that identifies a Flutter build tree rather than an install.
const _devBuildMarker = r'\build\windows\';

// Package identity cannot change during the process lifetime, so the probe
// runs once.
bool? _cachedIsMsix;

/// Detects whether the running process is packaged in an MSIX container.
///
/// Uses the package identity API rather than an environment variable: env vars
/// are inherited by child processes, so a link click handed down from another
/// MSIX app (the current Teams client, for one) used to be enough to produce a
/// false positive — and a false positive here disables registration entirely,
/// silently breaking link capture for a standalone install.
bool isRunningInMsix() {
  if (!Platform.isWindows) return false;
  return _cachedIsMsix ??= _hasPackageIdentity();
}

bool _hasPackageIdentity() {
  try {
    final getCurrentPackageFullName = DynamicLibrary.open('kernel32.dll')
        .lookupFunction<
          Int32 Function(Pointer<Uint32>, Pointer<Utf16>),
          int Function(Pointer<Uint32>, Pointer<Utf16>)
        >('GetCurrentPackageFullName');
    final length = calloc<Uint32>();
    try {
      // A zero-sized buffer answers the only question we care about: identity
      // present (ERROR_INSUFFICIENT_BUFFER) or absent (APPMODEL_ERROR_NO_PACKAGE).
      final rc = getCurrentPackageFullName(length, nullptr);
      return rc != _appmodelErrorNoPackage;
    } finally {
      calloc.free(length);
    }
  } on Object {
    // Symbol missing or FFI unavailable: fall back to the path heuristic.
    return Platform.resolvedExecutable.toLowerCase().contains(r'\windowsapps\');
  }
}

/// True when [executablePath] points inside a local Flutter build tree.
///
/// A build tree must never own the shell registration: the path vanishes as
/// soon as the tree is cleaned, moved or rebuilt elsewhere, and because
/// `HKCU\Software\Classes` shadows `HKLM`, that dead ProgId then hijacks link
/// handling from the real installation — Store or standalone alike.
bool isDevBuildPath(String executablePath) => executablePath
    .replaceAll('/', r'\')
    .toLowerCase()
    .contains(_devBuildMarker);

@visibleForTesting
void resetPackageContextCache() => _cachedIsMsix = null;
