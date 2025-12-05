import SwiftUI
import MapKit


@main
struct OpenStreetMapApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}


struct Coordinate: Hashable {
    let x: Int
    let y: Int
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
    func add(x: Int, y: Int) {
        let coord = Coordinate(x: x, y: y)
        let new = SensorItem(coordinate: coord)
        items.append(new)
        selection = new
    }

    // Updates coordinates of an existing sensor item
    func update(item: SensorItem, x: Int, y: Int) {
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

    var body: some View {
        // Two-column split: sidebar (list) + detail (map). Middle content column removed.
        NavigationSplitView {
            // Sidebar
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
                    .accessibilityLabel("Add item")
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
                    .accessibilityLabel("Edit selected item")
                }
            }
            .sheet(isPresented: $isPresentingAdd) {
                AddSensorView { x, y in
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
            // Detail – existing map view
            MapView()
                .ignoresSafeArea()
        }
    }
}

struct AddSensorView: View {
    @Environment(\.dismiss) private var dismiss

    @State private var xText: String = ""
    @State private var yText: String = ""

    private let viewTitle: String
    private let confirmTitle: String

    var onSave: (Int, Int) -> Void

    private var parsedX: Int? { Int(xText.trimmingCharacters(in: .whitespacesAndNewlines)) }
    private var parsedY: Int? { Int(yText.trimmingCharacters(in: .whitespacesAndNewlines)) }
    private var canSave: Bool { parsedX != nil && parsedY != nil }

    init(initialX: Int? = nil,
         initialY: Int? = nil,
         title: String = "Add Sensor",
         confirmLabel: String = "Save",
         onSave: @escaping (Int, Int) -> Void) {
        _xText = State(initialValue: initialX.map(String.init) ?? "")
        _yText = State(initialValue: initialY.map(String.init) ?? "")
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
    func makeUIView(context: Context) -> MKMapView {
        let mapView = MKMapView()
        
        // Configure OpenStreetMap Tile Overlay
        let template = "https://tile.openstreetmap.org/{z}/{x}/{y}.png"
        let overlay = MKTileOverlay(urlTemplate: template)
        overlay.canReplaceMapContent = true
        mapView.addOverlay(overlay, level: .aboveLabels)
        mapView.delegate = context.coordinator
        
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
    }
}
