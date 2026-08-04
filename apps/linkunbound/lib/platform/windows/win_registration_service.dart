import 'dart:ffi';

import 'package:flutter/foundation.dart';
import 'package:logging/logging.dart';
import 'package:linkunbound_core/linkunbound_core.dart';
import 'package:win32_registry/win32_registry.dart';

import '../local_file_url.dart';
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

/// Extensions registered under OpenWithProgIds during install.
/// Exposed for testing to verify write/remove symmetry without touching the registry.
@visibleForTesting
List<String> get winRegistrationOpenWithExts =>
    WinRegistrationService._openWithExts;

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

const _openCommandPath = r'Software\Classes\LinkUnboundURL\shell\open\command';

final class WinRegistrationService implements RegistrationService {
  @override
  Future<void> register(String executablePath) async {
    if (isRunningInMsix()) {
      // Protocol association is declared via the MSIX manifest; HKCU writes
      // are sandboxed to the package and invisible to the Shell.
      return;
    }
    if (isDevBuildPath(executablePath)) {
      _log.warning(
        'Refusing to register a local build tree as URL handler: '
        '${redactPath(executablePath)}',
      );
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
    _log.info('Registered URL handler at ${redactPath(exe)}');
  }

  /// Reconciles the recorded handler with the running installation.
  ///
  /// `register()` used to run only on first boot, so the handler path was
  /// frozen forever: updating, reinstalling or moving the app left HKCU
  /// pointing at an executable that no longer exists, and Windows silently
  /// stopped offering LinkUnbound as a browser. Because `HKCU\Software\Classes`
  /// shadows `HKLM`, a stale per-user entry also overrides a Store or
  /// standalone install — so the fix has to remove it, not just rewrite it.
  ///
  /// Safe to call on every launch: it only writes when something drifted.
  @override
  Future<void> ensureRegistered(String executablePath) async {
    final recorded = _readRegisteredCommand();

    if (isRunningInMsix()) {
      // The package manifest owns the association. Any per-user ProgId left
      // behind by a standalone install or a local build shadows it.
      if (recorded != null) {
        _log.warning(
          'Removing stale per-user registration shadowing the MSIX package '
          '(recorded=${redactPath(recorded)})',
        );
        _removeHkcuRegistration();
      }
      return;
    }

    if (isDevBuildPath(executablePath)) {
      // Never let a build tree own the association. If a previous run of this
      // same tree claimed it, drop it so the installed copy takes over.
      if (recorded != null && isDevBuildPath(recorded)) {
        _log.warning(
          'Removing registration owned by a local build tree '
          '(recorded=${redactPath(recorded)})',
        );
        _removeHkcuRegistration();
      } else {
        _log.info('Running from a local build tree; registration left intact');
      }
      return;
    }

    final expected = '"${executablePath.replaceAll('/', '\\')}" "%1"';
    if (recorded == expected) return;

    _log.info(
      'Handler command drifted; re-registering '
      '(recorded=${recorded == null ? '<none>' : redactPath(recorded)})',
    );
    await register(executablePath);
  }

  @override
  Future<void> unregister() async {
    if (isRunningInMsix()) {
      return;
    }
    _removeHkcuRegistration();
  }

  void _removeHkcuRegistration() {
    _deleteKeyTree(r'Software\Classes\LinkUnboundURL');
    _deleteKeyTree(r'Software\Clients\StartMenuInternet\LinkUnbound');
    _deleteKeyTree(r'Software\LinkUnbound');
    _removeRegisteredApplication();
    _removeOpenWithProgIds();
    _notifyShell();
  }

  /// The `shell\open\command` value currently recorded for our ProgId, or null
  /// when the app is not registered per-user.
  String? _readRegisteredCommand() {
    try {
      final key = Registry.openPath(
        RegistryHive.currentUser,
        path: _openCommandPath,
      );
      final command = key.getValueAsString('');
      key.close();
      return (command == null || command.isEmpty) ? null : command;
    } on Exception {
      return null;
    }
  }

  /// Intercepts the `microsoft-edge:` protocol so links opened from inside
  /// Microsoft apps reach the picker.
  ///
  /// Teams, Outlook, Widgets, Copilot and Start menu search do not open plain
  /// `https:` URLs — they wrap them in `microsoft-edge:`, a scheme hardwired to
  /// Edge that ignores the default-browser setting entirely. Capturing it is
  /// the only way those links can be offered a choice of browser, which is why
  /// `stripEdgeProtocol` already exists on the parsing side.
  ///
  /// Opt-in on purpose: it takes a protocol away from Edge, and a user who
  /// wants Edge's behaviour must be able to keep it.
  @override
  Future<void> setEdgeProtocolCapture(
    bool enabled,
    String executablePath,
  ) async {
    if (isRunningInMsix()) {
      // An MSIX package cannot claim a protocol another package owns.
      _log.info('Edge protocol capture unavailable in MSIX context');
      return;
    }
    if (enabled) {
      if (isDevBuildPath(executablePath)) {
        _log.warning('Refusing Edge protocol capture from a local build tree');
        return;
      }
      final quotedExe = '"${executablePath.replaceAll('/', '\\')}"';
      _writeEdgeProtocolProgId(quotedExe);
      _log.info('Edge protocol capture enabled');
    } else {
      _deleteKeyTree(r'Software\Classes\LinkUnboundEdgeProto');
      _deleteKeyTree(r'Software\Classes\microsoft-edge');
      _log.info('Edge protocol capture disabled');
    }
    _notifyShell();
  }

  @override
  Future<bool> get capturesEdgeProtocol async {
    if (isRunningInMsix()) return false;
    try {
      final key = Registry.openPath(
        RegistryHive.currentUser,
        path: r'Software\Classes\microsoft-edge\shell\open\command',
      );
      final command = key.getValueAsString('');
      key.close();
      return command != null && command.toLowerCase().contains('linkunbound');
    } on Exception {
      return false;
    }
  }

  void _writeEdgeProtocolProgId(String quotedExe) {
    final classes = Registry.openPath(
      RegistryHive.currentUser,
      path: r'Software\Classes',
      desiredAccessRights: AccessRights.allAccess,
    );

    for (final progId in ['LinkUnboundEdgeProto', 'microsoft-edge']) {
      final key = classes.createKey(progId);
      key.createValue(
        const RegistryValue('', RegistryValueType.string, 'LinkUnbound URL'),
      );
      // URL protocol keys are identified by this empty-valued marker.
      key.createValue(
        const RegistryValue('URL Protocol', RegistryValueType.string, ''),
      );
      final command = key.createKey(r'shell\open\command');
      command.createValue(
        RegistryValue('', RegistryValueType.string, '$quotedExe "%1"'),
      );
      command.close();
      key.close();
    }

    classes.close();
  }

  @override
  Future<HandlerDiagnostics> diagnose(String executablePath) async {
    final recorded = _readRegisteredCommand();
    final packaged = isRunningInMsix();
    final expected = '"${executablePath.replaceAll('/', '\\')}" "%1"';
    return HandlerDiagnostics(
      isDefaultBrowser: await isDefault,
      // Under MSIX the association lives in the package manifest, so having no
      // per-user command recorded is the correct state, not a fault.
      commandMatchesExecutable: packaged
          ? recorded == null
          : recorded == expected,
      runningFromDevBuild: isDevBuildPath(executablePath),
      isPackaged: packaged,
      recordedCommand: recorded,
    );
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

  static const _openWithExts = [
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

  void _writeOpenWithProgIds() {
    final classes = Registry.openPath(
      RegistryHive.currentUser,
      path: r'Software\Classes',
      desiredAccessRights: AccessRights.allAccess,
    );
    for (final ext in _openWithExts) {
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

  void _removeOpenWithProgIds() {
    for (final ext in _openWithExts) {
      try {
        final openWith = Registry.openPath(
          RegistryHive.currentUser,
          path: r'Software\Classes\' + ext + r'\OpenWithProgIds',
          desiredAccessRights: AccessRights.allAccess,
        );
        openWith.deleteValue('LinkUnboundURL');
        openWith.close();
      } on Exception catch (e) {
        _log.fine('Failed to remove OpenWithProgIds for $ext: $e');
      }
    }
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
