import 'dart:io';

import 'package:linkunbound_core/linkunbound_core.dart';

import 'cursor_locator.dart';
import 'tray_controller.dart';

abstract class PlatformBindings {
  BrowserDetector get browserDetector;
  IconExtractor get iconExtractor;
  RegistrationService get registrationService;
  StartupService get startupService;
  LaunchService get launchService;
  TrayController get trayController;
  CursorLocator get cursorLocator;

  /// Event derived from process arguments (Windows: argv URL; macOS: always null).
  InboundEvent? get initialEvent;

  /// Stream of inbound events from the OS once this process is the resident.
  /// Includes the [initialEvent] (if any) followed by subsequent OS events.
  Stream<InboundEvent> get inboundEvents;

  /// Try to forward [event] to an existing resident instance.
  /// Returns true if delegation succeeded and the caller should exit.
  Future<bool> tryDelegate(InboundEvent? event);

  /// Become the resident instance (single-instance lock + start listening).
  /// Returns false if another resident already exists and could not be reached.
  Future<bool> claim();

  /// Release single-instance resources.
  Future<void> release();

  String get executablePath;
  String get trayIconPath;

  bool get startsHidden;

  Directory get appDataDir;
  Directory get iconsDir;
  File get browsersFile;
  File get rulesFile;
  File get logFile;
  File get localeFile;
  File get themeFile;
  File get edgeWarningFile;
  File get hideTrayFile;
  File get globalHotkeyFile;
}
