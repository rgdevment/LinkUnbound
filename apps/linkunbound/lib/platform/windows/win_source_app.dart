import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';
import 'package:logging/logging.dart';

final _log = Logger('WinSourceApp');

const _th32csSnapProcess = 0x00000002;
const _processQueryLimitedInformation = 0x1000;
const _invalidHandleValue = -1;

/// Must match the `@Array` size of [_ProcessEntry32W.szExeFile].
const _maxPath = 260;

/// `QueryFullProcessImageNameW` may return an extended-length path.
const _maxLongPath = 32768;

/// US English + Unicode: the string block almost every signed binary ships.
const _defaultTranslation = '040904B0';

/// `PROCESSENTRY32W` from tlhelp32.h.
final class _ProcessEntry32W extends Struct {
  @Uint32()
  external int dwSize;
  @Uint32()
  external int cntUsage;
  @Uint32()
  external int th32ProcessID;
  @IntPtr()
  external int th32DefaultHeapID;
  @Uint32()
  external int th32ModuleID;
  @Uint32()
  external int cntThreads;
  @Uint32()
  external int th32ParentProcessID;
  @Int32()
  external int pcPriClassBase;
  @Uint32()
  external int dwFlags;
  @Array(260)
  external Array<Uint16> szExeFile;
}

final _kernel32 = DynamicLibrary.open('kernel32.dll');
final _versionDll = DynamicLibrary.open('version.dll');

final _getCurrentProcessId = _kernel32
    .lookupFunction<Uint32 Function(), int Function()>('GetCurrentProcessId');

final _createToolhelp32Snapshot = _kernel32
    .lookupFunction<IntPtr Function(Uint32, Uint32), int Function(int, int)>(
      'CreateToolhelp32Snapshot',
    );

final _process32First = _kernel32
    .lookupFunction<
      Int32 Function(IntPtr, Pointer<_ProcessEntry32W>),
      int Function(int, Pointer<_ProcessEntry32W>)
    >('Process32FirstW');

final _process32Next = _kernel32
    .lookupFunction<
      Int32 Function(IntPtr, Pointer<_ProcessEntry32W>),
      int Function(int, Pointer<_ProcessEntry32W>)
    >('Process32NextW');

final _openProcess = _kernel32
    .lookupFunction<
      IntPtr Function(Uint32, Int32, Uint32),
      int Function(int, int, int)
    >('OpenProcess');

final _queryFullProcessImageName = _kernel32
    .lookupFunction<
      Int32 Function(IntPtr, Uint32, Pointer<Utf16>, Pointer<Uint32>),
      int Function(int, int, Pointer<Utf16>, Pointer<Uint32>)
    >('QueryFullProcessImageNameW');

final _closeHandle = _kernel32
    .lookupFunction<Int32 Function(IntPtr), int Function(int)>('CloseHandle');

final _getFileVersionInfoSize = _versionDll
    .lookupFunction<
      Uint32 Function(Pointer<Utf16>, Pointer<Uint32>),
      int Function(Pointer<Utf16>, Pointer<Uint32>)
    >('GetFileVersionInfoSizeW');

final _getFileVersionInfo = _versionDll
    .lookupFunction<
      Int32 Function(Pointer<Utf16>, Uint32, Uint32, Pointer<Void>),
      int Function(Pointer<Utf16>, int, int, Pointer<Void>)
    >('GetFileVersionInfoW');

final _verQueryValue = _versionDll
    .lookupFunction<
      Int32 Function(
        Pointer<Void>,
        Pointer<Utf16>,
        Pointer<Pointer<Void>>,
        Pointer<Uint32>,
      ),
      int Function(
        Pointer<Void>,
        Pointer<Utf16>,
        Pointer<Pointer<Void>>,
        Pointer<Uint32>,
      )
    >('VerQueryValueW');

/// Executable name of the parent process, without path and without the `.exe`
/// extension, lowercased (e.g. `slack`, `teams`, `explorer`).
///
/// When Windows hands a link to this app, the process that called
/// `ShellExecute` is normally our parent, so this is the closest thing to
/// "which app did the click come from". Returns null when it cannot be
/// determined.
String? parentProcessName() {
  if (!Platform.isWindows) return null;
  try {
    final parent = _parentProcess();
    if (parent == null) return null;
    return _baseName(parent.exeFile);
  } on Object catch (e) {
    _log.fine('Could not resolve parent process name: $e');
    return null;
  }
}

/// Human readable name of the parent process for the UI: the executable's
/// `FileDescription` when the version resource exposes one (e.g. `Microsoft
/// Teams`), otherwise the capitalized executable name. Returns null when it
/// cannot be determined.
String? parentProcessDisplayName() {
  if (!Platform.isWindows) return null;
  try {
    final parent = _parentProcess();
    if (parent == null) return null;
    final name = _baseName(parent.exeFile);
    if (name == null) return null;

    final imagePath = _processImagePath(parent.pid);
    final description = imagePath == null ? null : _fileDescription(imagePath);
    return description ?? _capitalize(name);
  } on Object catch (e) {
    _log.fine('Could not resolve parent process display name: $e');
    return null;
  }
}

/// PID and executable name of the process that spawned this one.
///
/// Both lookups share one snapshot: the parent is read from our own entry and
/// then resolved in a second pass, so the parent must still be alive (or at
/// least still listed) when the snapshot is taken.
({int pid, String exeFile})? _parentProcess() {
  final snapshot = _createToolhelp32Snapshot(_th32csSnapProcess, 0);
  if (snapshot == 0 || snapshot == _invalidHandleValue) {
    _log.fine('CreateToolhelp32Snapshot failed');
    return null;
  }

  final entry = calloc<_ProcessEntry32W>();
  try {
    if (!_seek(snapshot, entry, _getCurrentProcessId())) return null;

    final parentPid = entry.ref.th32ParentProcessID;
    if (parentPid == 0) return null;

    if (!_seek(snapshot, entry, parentPid)) {
      _log.fine('Parent process $parentPid is no longer in the snapshot');
      return null;
    }
    return (pid: parentPid, exeFile: _exeFileOf(entry.ref));
  } finally {
    calloc.free(entry);
    _closeHandle(snapshot);
  }
}

/// Rewinds the snapshot and leaves [entry] on the process matching [pid].
bool _seek(int snapshot, Pointer<_ProcessEntry32W> entry, int pid) {
  // Process32FirstW rejects the entry unless dwSize is set, and it restarts
  // the walk, which is what lets one snapshot serve both passes.
  entry.ref.dwSize = sizeOf<_ProcessEntry32W>();

  var found = _process32First(snapshot, entry);
  while (found != 0) {
    if (entry.ref.th32ProcessID == pid) return true;
    found = _process32Next(snapshot, entry);
  }
  return false;
}

String _exeFileOf(_ProcessEntry32W entry) {
  final codes = <int>[];
  for (var i = 0; i < _maxPath; i++) {
    final code = entry.szExeFile[i];
    if (code == 0) break;
    codes.add(code);
  }
  return String.fromCharCodes(codes);
}

/// Full path of the image backing [pid], or null when it cannot be opened
/// (the process died, or it runs at a higher integrity level).
String? _processImagePath(int pid) {
  final handle = _openProcess(_processQueryLimitedInformation, 0, pid);
  if (handle == 0) return null;

  final buffer = calloc<Uint16>(_maxLongPath);
  final size = calloc<Uint32>();
  try {
    size.value = _maxLongPath;
    if (_queryFullProcessImageName(handle, 0, buffer.cast(), size) == 0) {
      return null;
    }
    return _readWide(buffer, size.value);
  } finally {
    calloc.free(buffer);
    calloc.free(size);
    _closeHandle(handle);
  }
}

/// `FileDescription` from the executable's version resource.
String? _fileDescription(String imagePath) {
  final pathPtr = imagePath.toNativeUtf16();
  try {
    final size = _getFileVersionInfoSize(pathPtr, nullptr);
    if (size == 0) return null;

    final block = calloc<Uint8>(size);
    try {
      if (_getFileVersionInfo(pathPtr, 0, size, block.cast()) == 0) return null;

      final direct = _queryDescription(block, _defaultTranslation);
      if (direct != null) return direct;

      // Localized builds ship a different language/codepage pair; the
      // translation table is the only way to know which one.
      final translation = _firstTranslation(block);
      if (translation == null) return null;
      if (translation == _defaultTranslation) return null;
      return _queryDescription(block, translation);
    } finally {
      calloc.free(block);
    }
  } finally {
    calloc.free(pathPtr);
  }
}

String? _queryDescription(Pointer<Uint8> block, String translation) {
  final subBlock = '\\StringFileInfo\\$translation\\FileDescription'
      .toNativeUtf16();
  final value = calloc<Pointer<Void>>();
  final length = calloc<Uint32>();
  try {
    if (_verQueryValue(block.cast(), subBlock, value, length) == 0) return null;
    if (value.value == nullptr || length.value == 0) return null;

    // puLen counts characters, terminator included.
    final text = _readWide(value.value.cast<Uint16>(), length.value).trim();
    return text.isEmpty ? null : text;
  } finally {
    calloc.free(subBlock);
    calloc.free(value);
    calloc.free(length);
  }
}

/// First `language|codepage` pair of the version resource, as the 8 hex digits
/// the `StringFileInfo` sub-block expects.
String? _firstTranslation(Pointer<Uint8> block) {
  final subBlock = r'\VarFileInfo\Translation'.toNativeUtf16();
  final value = calloc<Pointer<Void>>();
  final length = calloc<Uint32>();
  try {
    if (_verQueryValue(block.cast(), subBlock, value, length) == 0) return null;
    if (value.value == nullptr || length.value < 4) return null;

    final pair = value.value.cast<Uint16>();
    return '${_hex4(pair[0])}${_hex4(pair[1])}';
  } finally {
    calloc.free(subBlock);
    calloc.free(value);
    calloc.free(length);
  }
}

String _hex4(int value) =>
    value.toRadixString(16).padLeft(4, '0').toUpperCase();

/// Reads at most [maxChars] UTF-16 code units, stopping at the NUL terminator.
String _readWide(Pointer<Uint16> chars, int maxChars) {
  final codes = <int>[];
  for (var i = 0; i < maxChars; i++) {
    final code = chars[i];
    if (code == 0) break;
    codes.add(code);
  }
  return String.fromCharCodes(codes);
}

/// `Slack.exe` -> `slack`. Toolhelp reports a bare module name, but strip any
/// directory anyway so the value stays a stable rule key.
String? _baseName(String exeFile) {
  var name = exeFile.toLowerCase();
  final separator = name.lastIndexOf(RegExp(r'[\\/]'));
  if (separator >= 0) name = name.substring(separator + 1);
  if (name.endsWith('.exe')) name = name.substring(0, name.length - 4);
  return name.isEmpty ? null : name;
}

String _capitalize(String name) =>
    name.isEmpty ? name : name[0].toUpperCase() + name.substring(1);
