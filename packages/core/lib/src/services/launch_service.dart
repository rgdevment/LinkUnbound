abstract interface class LaunchService {
  /// Opens [url] in the browser at [executablePath].
  ///
  /// [privateArgs] is passed separately rather than folded into [extraArgs]
  /// because a private window is not merely an extra switch on macOS: `open`
  /// hands arguments to an already-running instance only when a new one is
  /// forced, so the platform layer has to know that this launch is private.
  /// Empty means a normal window.
  Future<void> launch(
    String executablePath,
    String url,
    List<String> extraArgs, {
    List<String> privateArgs = const [],
  });
}
