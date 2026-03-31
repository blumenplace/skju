import SwiftUI
import SwiftData
import OSLog


extension Logger {
    private static var subsystem = Bundle.main.bundleIdentifier!

    static let network = Logger(subsystem: subsystem, category: "Network")
    static let ui = Logger(subsystem: subsystem, category: "UI")
}

@main
struct SkjuSimulatorApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
        .modelContainer(
            for: StationItem.self,
            /*configurations: [
                SensorStoreConfiguration(
                    identifier: "SensorRemoteStore",
                    baseURL: URL(string: "https://skju-sim.blumen.place")!
                )
            ]*/
        )
        /*
        .commands {
            // Add custom menu commands
            CommandGroup(after: .newItem) {
                Button("New Sensor") {
                    // Action
                }
                .keyboardShortcut("n", modifiers: [.command, .shift])
            }
        }*/
    }
}
