import AppKit
import FlutterMacOS

final class BrowserDetectorChannel {
  static let channelName = "linkunbound/browser_detector"

  private let channel: FlutterMethodChannel
  private let ownBundleId: String

  init(messenger: FlutterBinaryMessenger) {
    channel = FlutterMethodChannel(name: Self.channelName, binaryMessenger: messenger)
    ownBundleId = Bundle.main.bundleIdentifier ?? "com.rgdevment.linkunbound"
    channel.setMethodCallHandler { [weak self] call, result in
      guard let self else { return result(FlutterMethodNotImplemented) }
      switch call.method {
      case "detect":
        result(self.detect())
      default:
        result(FlutterMethodNotImplemented)
      }
    }
  }

  private func detect() -> [[String: String]] {
    let workspace = NSWorkspace.shared
    var seen = Set<String>()
    var browsers: [[String: String]] = []

    var candidateURLs: [URL] = []
    if let httpsURL = URL(string: "https://example.com") {
      candidateURLs.append(contentsOf: workspace.urlsForApplications(toOpen: httpsURL))
    }

    let isPrimaryHttpHandler: (URL) -> Bool = { appURL in
      guard
        let info = Bundle(url: appURL)?.infoDictionary,
        let urlTypes = info["CFBundleURLTypes"] as? [[String: Any]]
      else {
        return false
      }
      for entry in urlTypes {
        let schemes = (entry["CFBundleURLSchemes"] as? [String] ?? []).map {
          $0.lowercased()
        }
        guard schemes.contains("http") || schemes.contains("https") else {
          continue
        }
        let rank = (entry["LSHandlerRank"] as? String)?.lowercased() ?? "default"
        return rank == "default" || rank == "owner"
      }
      return false
    }

    let allowedAppRoots: [String] = [
      "/Applications/",
      "/System/Applications/",
      "/System/Volumes/Preboot/",
      // Safari ships from a cryptex on macOS 13+, so without this it is
      // filtered out and users with no other browser get an empty picker.
      "/System/Cryptexes/App/System/Applications/",
      NSString("~/Applications/").expandingTildeInPath + "/",
    ]

    for appURL in candidateURLs {
      guard let bundle = Bundle(url: appURL),
            let bundleId = bundle.bundleIdentifier,
            bundleId != ownBundleId,
            isPrimaryHttpHandler(appURL),
            !seen.contains(bundleId)
      else { continue }

      let path = appURL.path
      guard allowedAppRoots.contains(where: { path.hasPrefix($0) }) else { continue }

      seen.insert(bundleId)

      let name = (bundle.infoDictionary?["CFBundleDisplayName"] as? String)
        ?? (bundle.infoDictionary?["CFBundleName"] as? String)
        ?? appURL.deletingPathExtension().lastPathComponent
      let id = bundleId
        .lowercased()
        .replacingOccurrences(of: ".", with: "-")

      browsers.append([
        "id": id,
        "name": name,
        "executablePath": appURL.path,
      ])
    }

    return browsers
  }
}
