import 'dart:ffi';

import 'package:ffi/ffi.dart';
import 'package:logging/logging.dart';

final _log = Logger('WinSecurity');

/// `SECURITY_ATTRIBUTES` as passed to `CreateNamedPipeW`.
final class SecurityAttributes extends Struct {
  @Uint32()
  external int nLength;
  external Pointer<Void> lpSecurityDescriptor;
  @Int32()
  external int bInheritHandle;
}

const _tokenQuery = 0x0008;
const _tokenUser = 1;
const _sddlRevision1 = 1;

final _advapi32 = DynamicLibrary.open('advapi32.dll');
final _kernel32 = DynamicLibrary.open('kernel32.dll');

final _getCurrentProcess = _kernel32
    .lookupFunction<IntPtr Function(), int Function()>('GetCurrentProcess');

final _localFree = _kernel32
    .lookupFunction<
      Pointer<Void> Function(Pointer<Void>),
      Pointer<Void> Function(Pointer<Void>)
    >('LocalFree');

final _openProcessToken = _advapi32
    .lookupFunction<
      Int32 Function(IntPtr, Uint32, Pointer<IntPtr>),
      int Function(int, int, Pointer<IntPtr>)
    >('OpenProcessToken');

final _getTokenInformation = _advapi32
    .lookupFunction<
      Int32 Function(IntPtr, Int32, Pointer<Void>, Uint32, Pointer<Uint32>),
      int Function(int, int, Pointer<Void>, int, Pointer<Uint32>)
    >('GetTokenInformation');

final _convertSidToStringSid = _advapi32
    .lookupFunction<
      Int32 Function(Pointer<Void>, Pointer<Pointer<Utf16>>),
      int Function(Pointer<Void>, Pointer<Pointer<Utf16>>)
    >('ConvertSidToStringSidW');

final _convertStringSdToSd = _advapi32
    .lookupFunction<
      Int32 Function(
        Pointer<Utf16>,
        Uint32,
        Pointer<Pointer<Void>>,
        Pointer<Uint32>,
      ),
      int Function(Pointer<Utf16>, int, Pointer<Pointer<Void>>, Pointer<Uint32>)
    >('ConvertStringSecurityDescriptorToSecurityDescriptorW');

final _closeHandle = _kernel32
    .lookupFunction<Int32 Function(IntPtr), int Function(int)>('CloseHandle');

/// Builds the security attributes used for the single-instance IPC pipe.
///
/// Two properties matter, and the default `NULL` descriptor gets both wrong:
///
///  * **Only this user.** The named pipe namespace is machine-wide, so with the
///    default DACL another signed-in user (fast user switching, RDS) could read
///    every URL this user opens.
///  * **Reachable from a lower integrity level.** The installer may launch the
///    app elevated, which puts the pipe at high integrity; a link clicked in
///    Slack arrives from a medium-integrity process, and `NO_WRITE_UP` on a low
///    label is what lets that process still deliver the URL.
///
/// Returns `nullptr` when the descriptor cannot be built; callers then fall
/// back to the default security, which is less safe but still functional.
Pointer<SecurityAttributes> buildPipeSecurityAttributes() {
  final sid = _currentUserSidString();
  if (sid == null) return nullptr;

  // GA = generic all for this user and SYSTEM; the low-integrity label with
  // NO_WRITE_UP keeps write access open to less privileged callers.
  final sddl = 'D:(A;;GA;;;$sid)(A;;GA;;;SY)S:(ML;;NW;;;LW)';
  final sddlPtr = sddl.toNativeUtf16();
  final descriptor = calloc<Pointer<Void>>();
  try {
    final ok = _convertStringSdToSd(
      sddlPtr,
      _sddlRevision1,
      descriptor,
      nullptr,
    );
    if (ok == 0) {
      _log.warning('Could not build pipe security descriptor');
      return nullptr;
    }
    final attrs = calloc<SecurityAttributes>();
    attrs.ref
      ..nLength = sizeOf<SecurityAttributes>()
      ..lpSecurityDescriptor = descriptor.value
      ..bInheritHandle = 0;
    return attrs;
  } finally {
    calloc.free(sddlPtr);
    calloc.free(descriptor);
  }
}

/// Frees the descriptor and the attributes allocated by
/// [buildPipeSecurityAttributes].
void freePipeSecurityAttributes(Pointer<SecurityAttributes> attrs) {
  if (attrs == nullptr) return;
  final descriptor = attrs.ref.lpSecurityDescriptor;
  if (descriptor != nullptr) _localFree(descriptor);
  calloc.free(attrs);
}

/// The current process user's SID in string form (`S-1-5-21-…`).
String? _currentUserSidString() {
  final tokenHandle = calloc<IntPtr>();
  try {
    if (_openProcessToken(_getCurrentProcess(), _tokenQuery, tokenHandle) ==
        0) {
      return null;
    }
    final token = tokenHandle.value;
    try {
      final needed = calloc<Uint32>();
      try {
        // First call sizes the buffer; it is expected to fail.
        _getTokenInformation(token, _tokenUser, nullptr, 0, needed);
        if (needed.value == 0) return null;
        final buffer = calloc<Uint8>(needed.value);
        try {
          final ok = _getTokenInformation(
            token,
            _tokenUser,
            buffer.cast(),
            needed.value,
            needed,
          );
          if (ok == 0) return null;
          // TOKEN_USER starts with SID_AND_ATTRIBUTES, whose first field is
          // the PSID we need.
          final sid = buffer.cast<Pointer<Void>>().value;
          final stringSid = calloc<Pointer<Utf16>>();
          try {
            if (_convertSidToStringSid(sid, stringSid) == 0) return null;
            final result = stringSid.value.toDartString();
            _localFree(stringSid.value.cast());
            return result;
          } finally {
            calloc.free(stringSid);
          }
        } finally {
          calloc.free(buffer);
        }
      } finally {
        calloc.free(needed);
      }
    } finally {
      _closeHandle(token);
    }
  } on Object catch (e) {
    _log.warning('Could not resolve current user SID: $e');
    return null;
  } finally {
    calloc.free(tokenHandle);
  }
}
