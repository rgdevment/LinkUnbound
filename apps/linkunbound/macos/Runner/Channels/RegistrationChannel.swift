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
        // Answer only once the system has actually applied (or refused) the
        // change, so Dart re-reads the real state instead of a stale one.
        self.setHandler(self.ownBundleId) { error in
          if let error {
            result(
              FlutterError(
                code: "registration_failed",
                message: error.localizedDescription,
                details: nil))
          } else {
            result(nil)
          }
        }
      case "unregister":
        // macOS has no "remove default" — fall back to Safari.
        self.setHandler(self.safariBundleId) { error in
          if let error {
            result(
              FlutterError(
                code: "unregistration_failed",
                message: error.localizedDescription,
                details: nil))
          } else {
            result(nil)
          }
        }
      case "isDefault":
        result(self.isDefault())
      case "defaultAssociations":
        result(self.defaultAssociations())
      default:
        result(FlutterMethodNotImplemented)
      }
    }
  }

  /// Points the given schemes at `bundleId`, reporting the first failure.
  ///
  /// For our own bundle the URL is always `Bundle.main.bundleURL`, never the
  /// Launch Services lookup: that lookup returns whichever copy LS happens to
  /// prefer, so with a debug build present it would register a path inside the
  /// build tree — and the association dies with the next `flutter clean`.
  private func setHandler(_ bundleId: String, completion: @escaping (Error?) -> Void) {
    let appURL: URL? =
      bundleId == ownBundleId
      ? Bundle.main.bundleURL
      : NSWorkspace.shared.urlForApplication(withBundleIdentifier: bundleId)

    guard let appURL else {
      completion(RegistrationError.applicationNotFound(bundleId))
      return
    }

    // macOS prompts the user for http/https and the answer can be "no"; the
    // errors used to be discarded, so Settings reported success either way.
    let group = DispatchGroup()
    var firstError: Error?
    for scheme in ["http", "https"] {
      group.enter()
      NSWorkspace.shared.setDefaultApplication(at: appURL, toOpenURLsWithScheme: scheme) { error in
        if let error, firstError == nil { firstError = error }
        group.leave()
      }
    }
    if let htmlType = UTType("public.html") {
      group.enter()
      Task {
        do {
          try await NSWorkspace.shared.setDefaultApplication(at: appURL, toOpen: htmlType)
        } catch {
          if firstError == nil { firstError = error }
        }
        group.leave()
      }
    }
    group.notify(queue: .main) { completion(firstError) }
  }

  enum RegistrationError: LocalizedError {
    case applicationNotFound(String)

    var errorDescription: String? {
      switch self {
      case .applicationNotFound(let bundleId):
        return "No application found for bundle identifier \(bundleId)"
      }
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
