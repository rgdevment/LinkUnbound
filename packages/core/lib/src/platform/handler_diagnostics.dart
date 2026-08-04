/// Snapshot of why link capture may not be working.
///
/// Exists because "LinkUnbound stopped opening my links" has several unrelated
/// causes that look identical to the user, and every one of them was previously
/// invisible without reading the registry by hand.
final class HandlerDiagnostics {
  const HandlerDiagnostics({
    required this.isDefaultBrowser,
    required this.commandMatchesExecutable,
    required this.runningFromDevBuild,
    required this.isPackaged,
    this.recordedCommand,
  });

  /// The OS reports this app as the handler for https.
  final bool isDefaultBrowser;

  /// The registered handler points at the executable currently running.
  /// False means links launch something else — typically a path left behind by
  /// an older install or a local build.
  final bool commandMatchesExecutable;

  /// This process runs from a build tree, so it must not own the registration.
  final bool runningFromDevBuild;

  /// Running from an MSIX/Store package, where the association comes from the
  /// package manifest rather than the registry.
  final bool isPackaged;

  /// The handler command as recorded by the OS, for display. Null when the app
  /// is not registered per-user, or on platforms without such a record.
  final String? recordedCommand;

  /// True when nothing needs fixing.
  bool get isHealthy =>
      isDefaultBrowser && (commandMatchesExecutable || isPackaged);

  /// True when re-registering would plausibly help. A build tree is excluded on
  /// purpose: registering it would make things worse, not better.
  bool get canRepair => !runningFromDevBuild && !commandMatchesExecutable;
}
