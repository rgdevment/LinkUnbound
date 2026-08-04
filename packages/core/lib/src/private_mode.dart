/// Command-line switches that open a browser directly in a private window.
///
/// There is no cross-browser standard here: each family invented its own
/// spelling, and a wrong switch is not ignored — Chromium treats an unknown
/// `--flag` as a URL-ish argument and Firefox opens an error page. So the
/// mapping is explicit and anything unrecognised opts out rather than guessing.
library;

/// Executable (or bundle) name fragments mapped to their private-mode switch.
///
/// Order matters: `msedge` must be tested before the generic Chromium list
/// because Edge is Chromium-based but spells the switch differently.
const _privateSwitches = <(List<String> markers, String flag)>[
  (['msedge', 'microsoft edge'], '-inprivate'),
  (['firefox', 'librewolf', 'waterfox', 'zen'], '-private-window'),
  (['opera'], '--private'),
  (
    ['chrome', 'chromium', 'brave', 'vivaldi', 'thorium', 'ungoogled', 'arc'],
    '--incognito',
  ),
];

/// Returns the arguments that make [executablePath] start in a private window,
/// or an empty list when the browser is not known to support it from the
/// command line (Safari, for one, offers no such switch).
List<String> privateModeArgs(String executablePath) {
  final needle = executablePath.toLowerCase();
  for (final (markers, flag) in _privateSwitches) {
    if (markers.any(needle.contains)) return [flag];
  }
  return const [];
}

/// Whether a private window can be requested for [executablePath].
bool supportsPrivateMode(String executablePath) =>
    privateModeArgs(executablePath).isNotEmpty;
