import Cocoa
import FlutterMacOS

final class WindowChannel {
  static let channelName = "linkunbound/window"

  private let channel: FlutterMethodChannel

  init(messenger: FlutterBinaryMessenger) {
    channel = FlutterMethodChannel(name: Self.channelName, binaryMessenger: messenger)
    channel.setMethodCallHandler { [weak self] call, result in
      guard self != nil else { return result(FlutterMethodNotImplemented) }
      switch call.method {
      case "setPickerMode":
        Self.applyPickerMode()
        result(nil)
      case "setSettingsMode":
        Self.applySettingsMode()
        result(nil)
      case "activate":
        Self.activate()
        result(nil)
      case "setRegular":
        NSApp.setActivationPolicy(.regular)
        result(nil)
      case "setAccessory":
        NSApp.setActivationPolicy(.accessory)
        result(nil)
      default:
        result(FlutterMethodNotImplemented)
      }
    }
  }

  /// The Flutter window, resolved through the delegate outlet.
  ///
  /// `NSApp.windows.first` was unreliable: the tray plugin creates an
  /// `NSStatusItem` whose backing window also lives in that array and the
  /// order is undocumented, so picker/settings styling and activation could be
  /// applied to the status bar window while the real one stayed hidden.
  private static func mainWindow() -> NSWindow? {
    (NSApp.delegate as? AppDelegate)?.mainFlutterWindow ?? NSApp.windows.first
  }

  private static func applyPickerMode() {
    guard let win = mainWindow() else { return }
    DispatchQueue.main.async {
      win.styleMask.remove(.resizable)
      win.styleMask.insert(.fullSizeContentView)
      win.titlebarAppearsTransparent = true
      win.titleVisibility = .hidden
      win.standardWindowButton(.closeButton)?.isHidden = true
      win.standardWindowButton(.miniaturizeButton)?.isHidden = true
      win.standardWindowButton(.zoomButton)?.isHidden = true
      win.level = .statusBar
    }
  }

  private static func applySettingsMode() {
    guard let win = mainWindow() else { return }
    DispatchQueue.main.async {
      // Settings uses the real native title bar so the close button sits in
      // its standard environment instead of floating over Flutter content.
      win.styleMask.insert(.resizable)
      win.styleMask.remove(.fullSizeContentView)
      win.titlebarAppearsTransparent = false
      win.titleVisibility = .visible
      win.title = "LinkUnbound"
      win.standardWindowButton(.closeButton)?.isHidden = false
      win.standardWindowButton(.miniaturizeButton)?.isHidden = true
      win.standardWindowButton(.zoomButton)?.isHidden = true
      win.level = .normal
    }
  }

  private static func activate() {
    guard let win = mainWindow() else { return }
    DispatchQueue.main.async {
      NSApp.activate(ignoringOtherApps: true)
      win.makeKeyAndOrderFront(nil)
    }
  }
}
