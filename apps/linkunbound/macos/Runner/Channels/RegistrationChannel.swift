import AppKit
import FlutterMacOS
import UniformTypeIdentifiers

/// `linkunbound/registration` — registers (or releases) the bundle as default
/// handler for http/https + public.html.
final class RegistrationChannel {
  static let channelName = "linkunbound/registration"

  private let channel: FlutterMethodChannel
  private let ownBundleId: String
  private let safariBundleId = "com.apple.Safari"

  init(messenger: FlutterBinaryMessenger) {
    channel = FlutterMethodChannel(name: Self.channelName, binaryMessenger: messenger)
    ownBundleId = Bundle.main.bundleIdentifier ?? "com.rgdevment.linkunbound"
    channel.setMethodCallHandler { [weak self] call, result in
      guard let self else { return result(FlutterMethodNotImplemented) }
      switch call.method {
      case "register":
        self.setHandler(self.ownBundleId)
        result(nil)
      case "unregister":
        // macOS has no "remove default" — fall back to Safari.
        self.setHandler(self.safariBundleId)
        result(nil)
      case "isDefault":
        result(self.isDefault())
      case "defaultAssociations":
        result(self.defaultAssociations())
      default:
        result(FlutterMethodNotImplemented)
      }
    }
  }

  private func setHandler(_ bundleId: String) {
    guard #available(macOS 12.0, *),
          let appURL = NSWorkspace.shared.urlForApplication(withBundleIdentifier: bundleId)
    else { return }

    NSWorkspace.shared.setDefaultApplication(at: appURL, toOpenURLsWithScheme: "http") { _ in }
    NSWorkspace.shared.setDefaultApplication(at: appURL, toOpenURLsWithScheme: "https") { _ in }
    if let htmlType = UTType("public.html") {
      // Fire-and-forget async API (no completion handler variant exists).
      Task { try? await NSWorkspace.shared.setDefaultApplication(at: appURL, toOpen: htmlType) }
    }
  }

  private func isDefault() -> Bool {
    let httpsHandler = handlerBundleId(forScheme: "https")
    return httpsHandler?.lowercased() == ownBundleId.lowercased()
  }

  private func defaultAssociations() -> [String] {
    var assoc: [String] = []
    if handlerBundleId(forScheme: "http")?.lowercased() == ownBundleId.lowercased() {
      assoc.append("http")
    }
    if handlerBundleId(forScheme: "https")?.lowercased() == ownBundleId.lowercased() {
      assoc.append("https")
    }
    if #available(macOS 12.0, *),
       let htmlType = UTType("public.html"),
       let htmlHandler = NSWorkspace.shared.urlForApplication(toOpen: htmlType),
       Bundle(url: htmlHandler)?.bundleIdentifier?.lowercased() == ownBundleId.lowercased() {
      assoc.append("public.html")
    }
    return assoc
  }

  private func handlerBundleId(forScheme scheme: String) -> String? {
    guard let url = URL(string: "\(scheme)://example.com"),
          let appURL = NSWorkspace.shared.urlForApplication(toOpen: url)
    else { return nil }
    return Bundle(url: appURL)?.bundleIdentifier
  }
}
