import 'handler_diagnostics.dart';

abstract interface class RegistrationService {
  /// Reports why link capture may be broken, so Settings can explain the
  /// problem and offer a one-click repair instead of leaving the user to
  /// inspect the registry.
  Future<HandlerDiagnostics> diagnose(String executablePath);

  Future<void> register(String executablePath);

  /// Reconciles the recorded registration with the running installation, and
  /// writes only when they differ. Called on every launch so a moved, updated
  /// or reinstalled app repairs its own handler instead of pointing at an
  /// executable that no longer exists.
  Future<void> ensureRegistered(String executablePath);

  Future<void> unregister();

  /// Enables or disables interception of the `microsoft-edge:` protocol, which
  /// Microsoft apps (Teams, Outlook, Widgets, Copilot, Start search) use for
  /// their links instead of plain `https:`. Windows-only; a no-op elsewhere.
  Future<void> setEdgeProtocolCapture(bool enabled, String executablePath);

  /// Whether `microsoft-edge:` links currently reach this app.
  Future<bool> get capturesEdgeProtocol;

  Future<bool> get isDefault;

  Future<Set<String>> get defaultAssociations;
}
