import SwiftUI
import MapKit
import UIKit


@main
struct OpenStreetMapApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}


struct Coordinate: Hashable {
    let x: Double
    let y: Double
}


struct SensorItem: Identifiable, Hashable {
    let id: UUID
    let coordinate: Coordinate

    init(id: UUID = UUID(), coordinate: Coordinate) {
        self.id = id
        self.coordinate = coordinate
    }

    var title: String { "x: \(coordinate.x), y: \(coordinate.y)" }
}


final class SensorsStore: ObservableObject {
    @Published var items: [SensorItem] = []
    @Published var selection: SensorItem? = nil

    // Adds a new sensor with the provided coordinates
    func add(x: Double, y: Double) {
        let coord = Coordinate(x: x, y: y)
        let new = SensorItem(coordinate: coord)
        items.append(new)
        selection = new
    }

    // Updates coordinates of an existing sensor item
    func update(item: SensorItem, x: Double, y: Double) {
        guard let index = items.firstIndex(where: { $0.id == item.id }) else { return }
        let updated = SensorItem(id: item.id, coordinate: Coordinate(x: x, y: y))
        items[index] = updated
        if selection?.id == item.id {
            selection = updated
        }
    }
}


struct ContentView: View {
    @StateObject private var store = SensorsStore()

    @State private var isPresentingAdd = false
    @State private var itemBeingEdited: SensorItem? = nil

    @State private var pendingInitialX: Double? = nil
    @State private var pendingInitialY: Double? = nil

    var body: some View {
        NavigationSplitView {
            List(store.items, selection: $store.selection) { item in
                Text(item.title)
                    .swipeActions {
                        Button("Edit") {
                            itemBeingEdited = item
                        }
                        .tint(.blue)
                    }
            }
            .navigationTitle("Items")
            .toolbar {
                ToolbarItem(placement: .primaryAction) {
                    Button {
                        isPresentingAdd = true
                    } label: {
                        Label("Add", systemImage: "plus")
                    }
                    .accessibilityLabel("Add a sensor")
                }
                ToolbarItem(placement: .primaryAction) {
                    Button {
                        if let selected = store.selection {
                            itemBeingEdited = selected
                        }
                    } label: {
                        Label("Edit", systemImage: "pencil")
                    }
                    .disabled(store.selection == nil)
                    .accessibilityLabel("Edit selected sensor")
                }
            }
            .sheet(isPresented: $isPresentingAdd, onDismiss: {
                pendingInitialX = nil
                pendingInitialY = nil
            }) {
                AddSensorView(
                    initialX: pendingInitialX,
                    initialY: pendingInitialY
                ) { x, y in
                    store.add(x: x, y: y)
                }
            }
            .sheet(item: $itemBeingEdited) { item in
                AddSensorView(
                    initialX: item.coordinate.x,
                    initialY: item.coordinate.y,
                    title: "Edit Sensor",
                    confirmLabel: "Update"
                ) { x, y in
                    store.update(item: item, x: x, y: y)
                }
            }
        } detail: {
            MapView { x, y in
                pendingInitialX = x
                pendingInitialY = y
                isPresentingAdd = true
            }.ignoresSafeArea()
        }
    }
}


struct AddSensorView: View {
    @Environment(\.dismiss) private var dismiss

    @State private var xText: String = ""
    @State private var yText: String = ""

    private let viewTitle: String
    private let confirmTitle: String

    var onSave: (Double, Double) -> Void

    private var parsedX: Double? { Double(xText.trimmingCharacters(in: .whitespacesAndNewlines)) }
    private var parsedY: Double? { Double(yText.trimmingCharacters(in: .whitespacesAndNewlines)) }
    private var canSave: Bool { parsedX != nil && parsedY != nil }

    init(initialX: Double? = nil,
         initialY: Double? = nil,
         title: String = "Add Sensor",
         confirmLabel: String = "Save",
         onSave: @escaping (Double, Double) -> Void)
    {
        _xText = State(initialValue: initialX.map{ xv in String(format: "%.5f", xv) } ?? "")
        _yText = State(initialValue: initialY.map{ yv in String(format: "%.5f", yv) } ?? "")
        self.viewTitle = title
        self.confirmTitle = confirmLabel
        self.onSave = onSave
    }

    var body: some View {
        NavigationView {
            Form {
                Section(header: Text("Coordinates")) {
                    TextField("X", text: $xText)
                        .keyboardType(.numbersAndPunctuation)
                        .textContentType(.oneTimeCode) // prevents iOS from offering contact info
                        .accessibilityLabel("X coordinate")
                    TextField("Y", text: $yText)
                        .keyboardType(.numbersAndPunctuation)
                        .textContentType(.oneTimeCode)
                        .accessibilityLabel("Y coordinate")
                }
            }
            .navigationTitle(viewTitle)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button(confirmTitle) {
                        if let x = parsedX, let y = parsedY {
                            onSave(x, y)
                            dismiss()
                        }
                    }
                    .disabled(!canSave)
                }
            }
        }
    }
}


struct MapView: UIViewRepresentable {
    var onLongPress: ((Double, Double) -> Void)? = nil

    func makeUIView(context: Context) -> MKMapView {
        let mapView = MKMapView()
        
        // Configure OpenStreetMap Tile Overlay
        let template = "https://tile.openstreetmap.org/{z}/{x}/{y}.png"
        let overlay = MKTileOverlay(urlTemplate: template)
        overlay.canReplaceMapContent = true
        mapView.addOverlay(overlay, level: .aboveLabels)
        mapView.delegate = context.coordinator
        
        // Add long press gesture recognizer
        let longPress = UILongPressGestureRecognizer(target: context.coordinator, action: #selector(Coordinator.handleLongPress(_:)))
        longPress.minimumPressDuration = 0.5
        mapView.addGestureRecognizer(longPress)
        
        return mapView
    }
    
    func updateUIView(_ uiView: MKMapView, context: Context) {}
    
    func makeCoordinator() -> Coordinator {
        Coordinator(self)
    }
    
    class Coordinator: NSObject, MKMapViewDelegate {
        var parent: MapView
        
        init(_ parent: MapView) {
            self.parent = parent
        }
        
        func mapView(_ mapView: MKMapView, rendererFor overlay: MKOverlay) -> MKOverlayRenderer {
            if let tileOverlay = overlay as? MKTileOverlay {
                return MKTileOverlayRenderer(tileOverlay: tileOverlay)
            }
            return MKOverlayRenderer(overlay: overlay)
        }

        @objc func handleLongPress(_ gesture: UILongPressGestureRecognizer) {
            guard gesture.state == .began, let view = gesture.view as? MKMapView else { return }
            let point = gesture.location(in: view)
            parent.onLongPress?(point.x, point.y)
        }
    }
}
