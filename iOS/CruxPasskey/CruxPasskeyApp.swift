import SwiftUI

enum Constants {
    static let domain = "crux-passkey-server-9uqexpm2.fermyon.app"
}

@main
struct CruxPasskeyApp: App {
  var body: some Scene {
    WindowGroup {
      ContentView(core: Core())
    }
  }
}
