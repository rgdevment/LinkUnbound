import 'dart:async';
import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'dart:isolate';

import 'package:ffi/ffi.dart';
import 'package:logging/logging.dart';
import 'package:linkunbound_core/linkunbound_core.dart';

import 'win_security.dart';

final _log = Logger('WinPipeServer');

const _pipeName = r'\\.\pipe\LinkUnbound';
const _bufferSize = 4096;

/// Sent by the isolate when the pipe name is already owned by someone else.
/// Distinct from the handle values, which are always positive.
const _pipeNameTakenSignal = -1;

const _pipeAccessDuplex = 0x00000003;
const _pipeTypeByte = 0x00000000;
const _pipeReadmodeByte = 0x00000000;
const _pipeWait = 0x00000000;
// Without this the pipe is reachable over SMB as \\<host>\pipe\LinkUnbound,
// letting a remote peer inject URLs into this session.
const _pipeRejectRemoteClients = 0x00000008;
// Makes CreateNamedPipeW fail if the name is already taken, so a squatter that
// grabbed it first is detected instead of silently receiving the user's URLs.
const _fileFlagFirstPipeInstance = 0x00080000;
const _pipeUnlimitedInstances = 255;
const _openExisting = 3;
const _genericWrite = 0x40000000;
const _invalidHandleValue = -1;
// Keeps a malicious or buggy server from impersonating this process when we
// connect as a client.
const _securitySqosPresent = 0x00100000;
const _securityIdentification = 0x00010000;
const _errorAccessDenied = 5;
const _errorPipeBusy = 231;

typedef _CreateNamedPipeWNative =
    IntPtr Function(
      Pointer<Utf16> lpName,
      Uint32 dwOpenMode,
      Uint32 dwPipeMode,
      Uint32 nMaxInstances,
      Uint32 nOutBufferSize,
      Uint32 nInBufferSize,
      Uint32 nDefaultTimeOut,
      Pointer<Void> lpSecurityAttributes,
    );
typedef _CreateNamedPipeWDart =
    int Function(
      Pointer<Utf16> lpName,
      int dwOpenMode,
      int dwPipeMode,
      int nMaxInstances,
      int nOutBufferSize,
      int nInBufferSize,
      int nDefaultTimeOut,
      Pointer<Void> lpSecurityAttributes,
    );

typedef _ConnectNamedPipeNative =
    Int32 Function(IntPtr hNamedPipe, Pointer<Void> lpOverlapped);
typedef _ConnectNamedPipeDart =
    int Function(int hNamedPipe, Pointer<Void> lpOverlapped);

typedef _DisconnectNamedPipeNative = Int32 Function(IntPtr hNamedPipe);
typedef _DisconnectNamedPipeDart = int Function(int hNamedPipe);

typedef _CreateFileWNative =
    IntPtr Function(
      Pointer<Utf16> lpFileName,
      Uint32 dwDesiredAccess,
      Uint32 dwShareMode,
      Pointer<Void> lpSecurityAttributes,
      Uint32 dwCreationDisposition,
      Uint32 dwFlagsAndAttributes,
      IntPtr hTemplateFile,
    );
typedef _CreateFileWDart =
    int Function(
      Pointer<Utf16> lpFileName,
      int dwDesiredAccess,
      int dwShareMode,
      Pointer<Void> lpSecurityAttributes,
      int dwCreationDisposition,
      int dwFlagsAndAttributes,
      int hTemplateFile,
    );

typedef _ReadFileNative =
    Int32 Function(
      IntPtr hFile,
      Pointer<Uint8> lpBuffer,
      Uint32 nNumberOfBytesToRead,
      Pointer<Uint32> lpNumberOfBytesRead,
      Pointer<Void> lpOverlapped,
    );
typedef _ReadFileDart =
    int Function(
      int hFile,
      Pointer<Uint8> lpBuffer,
      int nNumberOfBytesToRead,
      Pointer<Uint32> lpNumberOfBytesRead,
      Pointer<Void> lpOverlapped,
    );

typedef _WriteFileNative =
    Int32 Function(
      IntPtr hFile,
      Pointer<Uint8> lpBuffer,
      Uint32 nNumberOfBytesToWrite,
      Pointer<Uint32> lpNumberOfBytesWritten,
      Pointer<Void> lpOverlapped,
    );
typedef _WriteFileDart =
    int Function(
      int hFile,
      Pointer<Uint8> lpBuffer,
      int nNumberOfBytesToWrite,
      Pointer<Uint32> lpNumberOfBytesWritten,
      Pointer<Void> lpOverlapped,
    );

typedef _CloseHandleNative = Int32 Function(IntPtr hObject);
typedef _CloseHandleDart = int Function(int hObject);

typedef _CancelIoExNative =
    Int32 Function(IntPtr hFile, Pointer<Void> lpOverlapped);
typedef _CancelIoExDart = int Function(int hFile, Pointer<Void> lpOverlapped);

final class _NativePipe {
  _NativePipe._();

  static final _kernel32 = DynamicLibrary.open('kernel32.dll');

  static final createNamedPipe = _kernel32
      .lookupFunction<_CreateNamedPipeWNative, _CreateNamedPipeWDart>(
        'CreateNamedPipeW',
      );
  static final connectNamedPipe = _kernel32
      .lookupFunction<_ConnectNamedPipeNative, _ConnectNamedPipeDart>(
        'ConnectNamedPipe',
      );
  static final disconnectNamedPipe = _kernel32
      .lookupFunction<_DisconnectNamedPipeNative, _DisconnectNamedPipeDart>(
        'DisconnectNamedPipe',
      );
  static final createFile = _kernel32
      .lookupFunction<_CreateFileWNative, _CreateFileWDart>('CreateFileW');
  static final readFile = _kernel32
      .lookupFunction<_ReadFileNative, _ReadFileDart>('ReadFile');
  static final writeFile = _kernel32
      .lookupFunction<_WriteFileNative, _WriteFileDart>('WriteFile');
  static final closeHandle = _kernel32
      .lookupFunction<_CloseHandleNative, _CloseHandleDart>('CloseHandle');
  static final cancelIoEx = _kernel32
      .lookupFunction<_CancelIoExNative, _CancelIoExDart>('CancelIoEx');
}

final class WinPipeServer implements InboundEventServer {
  // Buffer events until the first listener attaches so a URL arriving between
  // claim() and bootstrap's stream.listen() is never silently dropped.
  final _controller = StreamController<InboundEvent>.broadcast();

  // _pendingBuffer holds events that arrive before the first listener.
  final _pendingBuffer = <InboundEvent>[];
  bool _hasListener = false;

  // Completes when the isolate signals it has created (and is blocking on)
  // its first named pipe — used by claim() to avoid the TOCTOU window.
  final _ready = Completer<void>();

  Isolate? _isolate;
  ReceivePort? _receivePort;
  int _pipeHandle = 0;

  // Guards against double-stop races (e.g. release() + dispose() in tests).
  bool _stopping = false;

  @override
  Stream<InboundEvent> get events {
    if (!_hasListener) {
      _hasListener = true;
      // Flush buffered events in order once a subscriber attaches.
      Future.microtask(() {
        for (final e in _pendingBuffer) {
          _controller.add(e);
        }
        _pendingBuffer.clear();
      });
    }
    return _controller.stream;
  }

  Future<void> get ready => _ready.future;

  /// Queues an event as if it had arrived over the pipe, honouring the same
  /// buffer-until-subscribed rule. Used for the URL carried by argv on launch.
  void pushEvent(InboundEvent event) {
    if (_hasListener) {
      _controller.add(event);
    } else {
      _pendingBuffer.add(event);
    }
  }

  @override
  Future<void> start() async {
    if (_isolate != null) return;

    _receivePort = ReceivePort();
    _receivePort!.listen((data) {
      if (data is String) {
        try {
          final event = InboundEvent.decode(data);
          if (_hasListener) {
            _controller.add(event);
          } else {
            _pendingBuffer.add(event);
          }
        } on FormatException catch (e) {
          _log.warning('Invalid inbound event: $e');
        }
      } else if (data == _pipeNameTakenSignal) {
        // Someone else owns \\.\pipe\LinkUnbound. Delegation from secondary
        // instances will not reach us; surface it instead of hanging on ready.
        _log.severe(
          'Pipe name already owned by another process: this instance cannot '
          'receive URLs from secondary instances',
        );
        if (!_ready.isCompleted) _ready.complete();
      } else if (data is int) {
        // The isolate reports each pipe handle when it starts listening and 0
        // right after closing it, so stop() never cancels a stale handle.
        _pipeHandle = data;
        if (data != 0 && !_ready.isCompleted) _ready.complete();
      }
    });

    _isolate = await Isolate.spawn(_serverLoop, _receivePort!.sendPort);
  }

  @override
  Future<void> stop() async {
    if (_stopping) return;
    _stopping = true;

    // Step 1: unblock any pending ConnectNamedPipe / ReadFile in the isolate.
    if (_pipeHandle != 0) {
      _NativePipe.cancelIoEx(_pipeHandle, nullptr);
    }

    // Step 2: kill the isolate and wait for it to actually exit before touching
    // the handle — closing the handle while the isolate still holds it causes
    // ERROR_INVALID_HANDLE and undefined behaviour during shutdown.
    final isolate = _isolate;
    if (isolate != null) {
      final exitPort = ReceivePort();
      isolate.addOnExitListener(exitPort.sendPort);
      isolate.kill(priority: Isolate.immediate);
      await exitPort.first
          .timeout(
            const Duration(seconds: 1),
            onTimeout: () => null, // best-effort; proceed regardless
          )
          .catchError((_) => null);
      exitPort.close();
      _isolate = null;
    }

    // Step 3: now it is safe to disconnect and close the handle.
    if (_pipeHandle != 0) {
      _NativePipe.disconnectNamedPipe(_pipeHandle);
      _NativePipe.closeHandle(_pipeHandle);
      _pipeHandle = 0;
    }

    _receivePort?.close();
    _receivePort = null;
    if (!_controller.isClosed) await _controller.close();
  }

  static void _serverLoop(SendPort sendPort) {
    final security = buildPipeSecurityAttributes();
    var firstInstance = true;
    while (true) {
      final pipeName = _pipeName.toNativeUtf16();
      final handle = _NativePipe.createNamedPipe(
        pipeName,
        _pipeAccessDuplex | (firstInstance ? _fileFlagFirstPipeInstance : 0),
        _pipeTypeByte |
            _pipeReadmodeByte |
            _pipeWait |
            _pipeRejectRemoteClients,
        _pipeUnlimitedInstances,
        _bufferSize,
        _bufferSize,
        0,
        security.cast(),
      );
      calloc.free(pipeName);

      if (handle == _invalidHandleValue) {
        final lastError = _getLastError();
        if (firstInstance &&
            (lastError == _errorAccessDenied || lastError == _errorPipeBusy)) {
          // Another process owns the name. Retrying forever would hand every
          // URL to it, so report and stop serving.
          sendPort.send(_pipeNameTakenSignal);
          return;
        }
        // Back off before retrying to avoid spinning the CPU on persistent errors.
        sleep(const Duration(milliseconds: 50));
        continue;
      }
      firstInstance = false;

      // Signal the main isolate: pipe is created and we're about to block on
      // ConnectNamedPipe — this unblocks claim()'s readiness await.
      sendPort.send(handle);

      final connected = _NativePipe.connectNamedPipe(handle, nullptr);
      if (connected == 0) {
        final lastError = _getLastError();
        if (lastError != 535) {
          _NativePipe.closeHandle(handle);
          sendPort.send(0);
          // Back off before retrying to avoid spinning the CPU on persistent errors.
          sleep(const Duration(milliseconds: 50));
          continue;
        }
      }

      final buffer = calloc<Uint8>(_bufferSize);
      final bytesRead = calloc<Uint32>();

      try {
        final success = _NativePipe.readFile(
          handle,
          buffer,
          _bufferSize - 1,
          bytesRead,
          nullptr,
        );

        if (success != 0 && bytesRead.value > 0) {
          final data = utf8.decode(
            buffer.asTypedList(bytesRead.value),
            allowMalformed: true,
          );
          sendPort.send(data);
        }
      } finally {
        calloc.free(buffer);
        calloc.free(bytesRead);
        _NativePipe.disconnectNamedPipe(handle);
        _NativePipe.closeHandle(handle);
        sendPort.send(0);
      }
    }
  }

  // Cached at class level so each iteration of _serverLoop doesn't reopen the DLL.
  static final _getLastError = () {
    final kernel32 = DynamicLibrary.open('kernel32.dll');
    return kernel32.lookupFunction<Uint32 Function(), int Function()>(
      'GetLastError',
    );
  }();
}

final class WinPipeClient implements InboundEventClient {
  @override
  Future<bool> send(InboundEvent event) async {
    final pipeName = _pipeName.toNativeUtf16();
    final handle = _NativePipe.createFile(
      pipeName,
      // Write-only: the client pushes one event and never reads a response.
      _genericWrite,
      0,
      nullptr,
      _openExisting,
      // Named pipes default to SecurityImpersonation, which would let whoever
      // is listening act as this user. Identification lets the server check
      // who we are without being able to impersonate us.
      _securitySqosPresent | _securityIdentification,
      0,
    );
    calloc.free(pipeName);

    if (handle == _invalidHandleValue) {
      _log.fine('Could not connect to pipe (no server running)');
      return false;
    }

    try {
      final data = utf8.encode(event.encode());
      final buffer = calloc<Uint8>(data.length);
      for (var i = 0; i < data.length; i++) {
        buffer[i] = data[i];
      }
      final bytesWritten = calloc<Uint32>();

      try {
        final success = _NativePipe.writeFile(
          handle,
          buffer,
          data.length,
          bytesWritten,
          nullptr,
        );
        return success != 0;
      } finally {
        calloc.free(buffer);
        calloc.free(bytesWritten);
      }
    } finally {
      _NativePipe.closeHandle(handle);
    }
  }
}
