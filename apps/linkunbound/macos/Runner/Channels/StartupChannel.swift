import Cocoa
import FlutterMacOS
import ServiceManagement

final class StartupChannel {
  static let channelName = "linkunbound/startup"

  private let channel: FlutterMethodChannel

  init(messenger: FlutterBinaryMessenger) {
    channel = FlutterMethodChannel(name: Self.channelName, binaryMessenger: messenger)
    channel.setMethodCallHandler { [weak self] call, result in
      guard self != nil else { return result(FlutterMethodNotImplemented) }
      switch call.method {
      case "enable":
        do {
          try SMAppService.mainApp.register()
          result(nil)
        } catch {
          result(FlutterError(code: "register_failed",
                              message: error.localizedDescription,
                              details: nil))
        }
      case "disable":
        do {
          try SMAppService.mainApp.unregister()
          result(nil)
        } catch {
          result(FlutterError(code: "unregister_failed",
                              message: error.localizedDescription,
                              details: nil))
        }
      case "isEnabled":
        result(SMAppService.mainApp.status == .enabled)
      case "isLoginItemLaunch":
        result((NSApp.delegate as? AppDelegate)?.launchedAsLoginItem ?? false)
      default:
        result(FlutterMethodNotImplemented)
      }
    }
  }
}
