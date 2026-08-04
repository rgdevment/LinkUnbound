import 'dart:io';

import 'package:linkunbound_core/linkunbound_core.dart';

/// Launches a browser via `/usr/bin/open`.
///
/// `executablePath` is the absolute path to the `.app` bundle (as returned by
/// `MacBrowserDetector`). `extraArgs` are forwarded as program arguments via
/// `--args`. The child process is detached so closing LinkUnbound does not kill
/// the browser.
class MacLaunchService implements LaunchService {
  @override
  Future<void> launch(
    String executablePath,
    String url,
    List<String> extraArgs, {
    List<String> privateArgs = const [],
  }) async {
    // Last line of defence before a process is spawned: a value starting with
    // `-` would be consumed by `open` as one of its own flags rather than
    // treated as the document to open.
    if (!isLaunchableUrl(url)) {
      throw ArgumentError.value(url, 'url', 'Not a launchable URL');
    }

    final List<String> args;
    if (privateArgs.isEmpty) {
      // `open` requires the document/URL BEFORE `--args`; everything after
      // `--args` is forwarded as argv to the launched app.
      args = <String>['-a', executablePath, url];
      if (extraArgs.isNotEmpty) {
        args.add('--args');
        args.addAll(extraArgs);
      }
    } else {
      // A private window needs `-n`: `open` drops `--args` entirely when the
      // app is already running, so without forcing a new instance the switch
      // would be silently ignored and the link would open in a normal window.
      // The URL then has to travel after `--args` as well, since the browser
      // itself — not `open` — is what must act on both.
      args = <String>[
        '-na',
        executablePath,
        '--args',
        ...extraArgs,
        ...privateArgs,
        url,
      ];
    }
    await Process.start('/usr/bin/open', args, mode: ProcessStartMode.detached);
  }
}
