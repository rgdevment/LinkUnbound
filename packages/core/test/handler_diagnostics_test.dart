import 'package:linkunbound_core/linkunbound_core.dart';
import 'package:test/test.dart';

HandlerDiagnostics diagnostics({
  bool isDefaultBrowser = true,
  bool commandMatchesExecutable = true,
  bool runningFromDevBuild = false,
  bool isPackaged = false,
  String? recordedCommand,
}) => HandlerDiagnostics(
  isDefaultBrowser: isDefaultBrowser,
  commandMatchesExecutable: commandMatchesExecutable,
  runningFromDevBuild: runningFromDevBuild,
  isPackaged: isPackaged,
  recordedCommand: recordedCommand,
);

void main() {
  group('isHealthy', () {
    test('is true when registered and the command points at this build', () {
      expect(diagnostics().isHealthy, isTrue);
    });

    test('is false when the app is not the default handler', () {
      expect(diagnostics(isDefaultBrowser: false).isHealthy, isFalse);
    });

    test('is false when the recorded command points somewhere else', () {
      expect(diagnostics(commandMatchesExecutable: false).isHealthy, isFalse);
    });

    test('ignores the command mismatch under MSIX', () {
      // The association comes from the package manifest, so there is no
      // per-user command to match against and a mismatch means nothing.
      expect(
        diagnostics(
          commandMatchesExecutable: false,
          isPackaged: true,
        ).isHealthy,
        isTrue,
      );
    });

    test('is false under MSIX when the app is not the default handler', () {
      expect(
        diagnostics(isDefaultBrowser: false, isPackaged: true).isHealthy,
        isFalse,
      );
    });
  });

  group('canRepair', () {
    test('is true when the recorded command drifted', () {
      expect(diagnostics(commandMatchesExecutable: false).canRepair, isTrue);
    });

    test('is false when there is nothing to repair', () {
      expect(diagnostics().canRepair, isFalse);
    });

    test('is false from a build tree even though the command drifted', () {
      // Registering a build tree makes things worse: the path disappears on
      // the next `flutter clean` and takes link capture with it.
      expect(
        diagnostics(
          commandMatchesExecutable: false,
          runningFromDevBuild: true,
        ).canRepair,
        isFalse,
      );
    });
  });

  test('recordedCommand is carried through for display', () {
    final d = diagnostics(recordedCommand: r'"C:\old\linkunbound.exe" "%1"');
    expect(d.recordedCommand, r'"C:\old\linkunbound.exe" "%1"');
    expect(diagnostics().recordedCommand, isNull);
  });
}
