import 'dart:ffi';

import 'package:flutter/foundation.dart';
import 'package:logging/logging.dart';
import 'package:linkunbound_core/linkunbound_core.dart';
import 'package:win32_registry/win32_registry.dart';

import 'win_package_context.dart';

final _log = Logger('WinRegistrationService');

// ProgIds written by _writeProgId / _writeCapabilities in the desktop install,
// stored lower-cased for O(1) case-insensitive lookup.
const _ownedProgIds = {'linkunboundurl', 'linkunboundedgeproto'};

/// Returns true when [progId] exactly matches one of the ProgIds the app
/// writes — case-insensitive. Substring match is intentionally avoided to
/// prevent false positives from third-party ProgIds that embed "linkunbound".
@visibleForTesting
bool progIdMatchesLinkUnbound(String? progId) {
  if (progId == null || progId.isEmpty) return false;
  return _ownedProgIds.contains(progId.toLowerCase());
}

/// Keys checked in the per-user registry when resolving default associations.
/// Exposed for testing.
@visibleForTesting
Iterable<String> get winRegistrationUserChoiceKeys =>
    WinRegistrationService._userChoicePaths.keys;

typedef _SHChangeNotifyNative =
    Void Function(
      Int32 wEventId,
      Uint32 uFlags,
      Pointer<Void> dwItem1,
      Pointer<Void> dwItem2,
    );
typedef _SHChangeNotifyDart =
    void Function(
      int wEventId,
      int uFlags,
      Pointer<Void> dwItem1,
      Pointer<Void> dwItem2,
    );

const _shcneAssocChanged = 0x08000000;
const _shcnfIdList = 0x0000;

// Loaded once per module; shell32 is always present on Windows.
final _SHChangeNotifyDart _shChangeNotify = DynamicLibrary.open(
  'shell32.dll',
).lookupFunction<_SHChangeNotifyNative, _SHChangeNotifyDart>('SHChangeNotify');

final class WinRegistrationService implements RegistrationService {
  @override
  Future<void> register(String executablePath) async {
    if (isRunningInMsix()) {
      // Protocol association is declared via the MSIX manifest; HKCU writes
      // are sandboxed to the package and invisible to the Shell.
      return;
    }
    final exe = executablePath.replaceAll('/', '\\');
    final quotedExe = '"$exe"';

    _writeProgId(exe, quotedExe);
    _writeStartMenuInternet(exe, quotedExe);
    _writeCapabilities(exe, quotedExe);
    _writeOpenWithProgIds();
    _writeRegisteredApplications();
    _notifyShell();
  }

  @override
  Future<void> unregister() async {
    if (isRunningInMsix()) {
      return;
    }
    _deleteKeyTree(r'Software\Classes\LinkUnboundURL');
    _deleteKeyTree(r'Software\Clients\StartMenuInternet\LinkUnbound');
    _deleteKeyTree(r'Software\LinkUnbound');
    _removeRegisteredApplication();
    _notifyShell();
  }

  @override
  Future<bool> get isDefault async {
    return _progIdBelongsToUs(
      _readUserChoiceProgId(_userChoicePaths['https']!),
    );
  }

  @override
  Future<Set<String>> get defaultAssociations async {
    final result = <String>{};
    for (final entry in _userChoicePaths.entries) {
      final progId = _readUserChoiceProgId(entry.value);
      if (_progIdBelongsToUs(progId)) {
        result.add(entry.key);
      }
    }
    return result;
  }

  String? _readUserChoiceProgId(String path) {
    try {
      final key = Registry.openPath(RegistryHive.currentUser, path: path);
      final progId = key.getValueAsString('ProgId');
      key.close();
      return progId;
    } on Exception {
      return null;
    }
  }

  // Desktop install writes "LinkUnboundURL". MSIX writes a package-scoped
  // ProgId of the form "AppX<base32-sha1-hash>" that does NOT embed the
  // identity name, so we resolve it via the class registration which carries
  // our AppUserModelID (containing the package family name).
  bool _progIdBelongsToUs(String? progId) {
    if (progIdMatchesLinkUnbound(progId)) return true;
    try {
      final key = Registry.openPath(
        RegistryHive.currentUser,
        path:
            r'Software\Classes\'
            '$progId'
            r'\Application',
      );
      final aumid = key.getValueAsString('AppUserModelID');
      key.close();
      if (aumid != null && aumid.toLowerCase().contains('linkunbound')) {
        return true;
      }
    } on Exception {
      // Class entry not found or no AppUserModelID — not ours.
    }
    return false;
  }

  static const _userChoicePaths = {
    'http':
        r'Software\Microsoft\Windows\Shell\Associations\UrlAssociations\http\UserChoice',
    'https':
        r'Software\Microsoft\Windows\Shell\Associations\UrlAssociations\https\UserChoice',
    '.htm':
        r'Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.htm\UserChoice',
    '.html':
        r'Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.html\UserChoice',
    '.xhtml':
        r'Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.xhtml\UserChoice',
    '.svg':
        r'Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.svg\UserChoice',
    '.pdf':
        r'Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.pdf\UserChoice',
  };

  void _writeProgId(String exe, String quotedExe) {
    final root = Registry.openPath(
      RegistryHive.currentUser,
      path: r'Software\Classes',
      desiredAccessRights: AccessRights.allAccess,
    );

    final progId = root.createKey('LinkUnboundURL');
    progId.createValue(
      const RegistryValue('', RegistryValueType.string, 'LinkUnbound URL'),
    );
    progId.createValue(
      const RegistryValue(
        'FriendlyTypeName',
        RegistryValueType.string,
        'LinkUnbound URL',
      ),
    );
    progId.createValue(
      const RegistryValue('EditFlags', RegistryValueType.int32, 2),
    );

    final app = progId.createKey('Application');
    app.createValue(
      const RegistryValue(
        'ApplicationName',
        RegistryValueType.string,
        'LinkUnbound',
      ),
    );
    app.createValue(
      const RegistryValue(
        'ApplicationDescription',
        RegistryValueType.string,
        'Browser picker for Windows',
      ),
    );
    app.createValue(
      RegistryValue(
        'ApplicationIcon',
        RegistryValueType.string,
        '$quotedExe,0',
      ),
    );
    app.close();

    final defaultIcon = progId.createKey('DefaultIcon');
    defaultIcon.createValue(
      RegistryValue('', RegistryValueType.string, '$quotedExe,0'),
    );
    defaultIcon.close();

    final command = progId.createKey(r'shell\open\command');
    command.createValue(
      RegistryValue('', RegistryValueType.string, '$quotedExe "%1"'),
    );
    command.close();

    progId.close();
    root.close();
  }

  void _writeStartMenuInternet(String exe, String quotedExe) {
    final root = Registry.openPath(
      RegistryHive.currentUser,
      path: r'Software\Clients\StartMenuInternet',
      desiredAccessRights: AccessRights.allAccess,
    );

    final key = root.createKey('LinkUnbound');
    key.createValue(
      const RegistryValue('', RegistryValueType.string, 'LinkUnbound'),
    );

    final defaultIcon = key.createKey('DefaultIcon');
    defaultIcon.createValue(
      RegistryValue('', RegistryValueType.string, '$quotedExe,0'),
    );
    defaultIcon.close();

    final command = key.createKey(r'shell\open\command');
    command.createValue(RegistryValue('', RegistryValueType.string, quotedExe));
    command.close();

    final installInfo = key.createKey('InstallInfo');
    installInfo.createValue(
      RegistryValue('ReinstallCommand', RegistryValueType.string, quotedExe),
    );
    installInfo.createValue(
      const RegistryValue('IconsVisible', RegistryValueType.int32, 1),
    );
    installInfo.close();

    key.close();
    root.close();
  }

  void _writeCapabilities(String exe, String quotedExe) {
    final root = Registry.openPath(
      RegistryHive.currentUser,
      path: r'Software',
      desiredAccessRights: AccessRights.allAccess,
    );

    final caps = root.createKey(r'LinkUnbound\Capabilities');
    caps.createValue(
      const RegistryValue(
        'ApplicationName',
        RegistryValueType.string,
        'LinkUnbound',
      ),
    );
    caps.createValue(
      const RegistryValue(
        'ApplicationDescription',
        RegistryValueType.string,
        'Browser picker for Windows',
      ),
    );
    caps.createValue(
      RegistryValue(
        'ApplicationIcon',
        RegistryValueType.string,
        '$quotedExe,0',
      ),
    );

    final startMenu = caps.createKey('Startmenu');
    startMenu.createValue(
      const RegistryValue(
        'StartMenuInternet',
        RegistryValueType.string,
        'LinkUnbound',
      ),
    );
    startMenu.close();

    final urlAssoc = caps.createKey('URLAssociations');
    urlAssoc.createValue(
      const RegistryValue('http', RegistryValueType.string, 'LinkUnboundURL'),
    );
    urlAssoc.createValue(
      const RegistryValue('https', RegistryValueType.string, 'LinkUnboundURL'),
    );
    urlAssoc.close();

    final fileAssoc = caps.createKey('FileAssociations');
    for (final ext in [
      '.htm',
      '.html',
      '.pdf',
      '.mhtml',
      '.mht',
      '.shtml',
      '.xhtml',
      '.xht',
      '.svg',
      '.webp',
    ]) {
      fileAssoc.createValue(
        RegistryValue(ext, RegistryValueType.string, 'LinkUnboundURL'),
      );
    }
    fileAssoc.close();

    caps.close();
    root.close();
  }

  void _writeOpenWithProgIds() {
    const exts = [
      '.htm',
      '.html',
      '.xhtml',
      '.xht',
      '.pdf',
      '.svg',
      '.mhtml',
      '.mht',
      '.shtml',
      '.webp',
    ];
    final classes = Registry.openPath(
      RegistryHive.currentUser,
      path: r'Software\Classes',
      desiredAccessRights: AccessRights.allAccess,
    );
    for (final ext in exts) {
      try {
        final extKey = classes.createKey(ext);
        final openWith = extKey.createKey('OpenWithProgIds');
        openWith.createValue(
          const RegistryValue('LinkUnboundURL', RegistryValueType.string, ''),
        );
        openWith.close();
        extKey.close();
      } on Exception catch (e) {
        _log.fine('Failed to write OpenWithProgIds for $ext: $e');
      }
    }
    classes.close();
  }

  void _writeRegisteredApplications() {
    final key = Registry.openPath(
      RegistryHive.currentUser,
      path: r'Software\RegisteredApplications',
      desiredAccessRights: AccessRights.allAccess,
    );
    key.createValue(
      const RegistryValue(
        'LinkUnbound',
        RegistryValueType.string,
        r'Software\LinkUnbound\Capabilities',
      ),
    );
    key.close();
  }

  void _removeRegisteredApplication() {
    try {
      final key = Registry.openPath(
        RegistryHive.currentUser,
        path: r'Software\RegisteredApplications',
        desiredAccessRights: AccessRights.allAccess,
      );
      key.deleteValue('LinkUnbound');
      key.close();
    } on Exception {
      // Value may not exist
    }
  }

  void _deleteKeyTree(String path) {
    try {
      final parent = Registry.openPath(
        RegistryHive.currentUser,
        path: _parentPath(path),
        desiredAccessRights: AccessRights.allAccess,
      );
      parent.deleteKey(_leafName(path), recursive: true);
      parent.close();
    } on Exception catch (e) {
      _log.fine('Key $path not found during unregister: $e');
    }
  }

  String _parentPath(String path) {
    final lastSlash = path.lastIndexOf('\\');
    return lastSlash < 0 ? '' : path.substring(0, lastSlash);
  }

  String _leafName(String path) {
    final lastSlash = path.lastIndexOf('\\');
    return lastSlash < 0 ? path : path.substring(lastSlash + 1);
  }

  void _notifyShell() {
    _shChangeNotify(_shcneAssocChanged, _shcnfIdList, nullptr, nullptr);
  }
}
