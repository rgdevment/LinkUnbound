import 'dart:io';

import 'package:linkunbound_core/linkunbound_core.dart';

final class WinLaunchService implements LaunchService {
  @override
  Future<void> launch(
    String executablePath,
    String url,
    List<String> extraArgs, {
    List<String> privateArgs = const [],
  }) async {
    // Last line of defence before a process is spawned: browsers read any
    // argument starting with `-` (or `/` on Windows) as a switch, and switches
    // like --gpu-launcher run arbitrary binaries.
    if (!isLaunchableUrl(url)) {
      throw ArgumentError.value(url, 'url', 'Not a launchable URL');
    }
    final args = [...extraArgs, ...privateArgs, url];
    await Process.start(executablePath, args, mode: ProcessStartMode.detached);
  }
}
