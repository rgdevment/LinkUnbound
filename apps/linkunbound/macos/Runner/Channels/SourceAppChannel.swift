import AppKit
import FlutterMacOS

/// `linkunbound/source_app` — best-effort origin of an inbound link.
///
/// macOS gives no originator for open events, so the application that owns the
/// foreground at that instant is used as an approximation.
final class SourceAppChannel {
  static let channelName = "linkunbound/source_app"

  private let channel: FlutterMethodChannel
  private let ownBundleId: String

  init(messenger: FlutterBinaryMessenger) {
    channel = FlutterMethodChannel(name: Self.channelName, binaryMessenger: messenger)
    ownBundleId = Bundle.main.bundleIdentifier ?? "com.rgdevment.linkunbound"
    channel.setMethodCallHandler { [weak self] call, result in
      guard let self else { return result(FlutterMethodNotImplemented) }
      switch call.method {
      case "frontmostApp":
        result(self.frontmostApp())
      default:
        result(FlutterMethodNotImplemented)
      }
    }
  }

  /// The foreground application, or `nil` when it cannot stand in for the origin.
  private func frontmostApp() -> [String: String]? {
    guard let app = NSWorkspace.shared.frontmostApplication,
          let bundleId = app.bundleIdentifier?.lowercased(),
          let name = app.localizedName
    else { return nil }

    // Ourselves in the foreground says nothing about where the link came from:
    // reporting it would let a rule match on LinkUnbound instead of on Slack,
    // Teams or whatever actually handed us the URL.
    guard bundleId != ownBundleId.lowercased() else { return nil }

    return ["id": bundleId, "name": name]
  }
}
