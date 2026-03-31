import MapKit
import SwiftData
import SwiftUI
import UIKit


struct NewSeismicStation: Identifiable {
    typealias ID = String

    var id: Self.ID { name }

    public let name: String
    public let coordinate: CLLocationCoordinate2D
    
    init(name: String, coordinate: CLLocationCoordinate2D) {
        self.name = name
        self.coordinate = coordinate
    }
}


struct ContentView: View {
    @Environment(\.modelContext) private var modelContext
    @Query(sort: [SortDescriptor(\SensorItem.x), SortDescriptor(\SensorItem.y)]) private var items: [SensorItem]

    @State private var selection: SensorItem? = nil

    @State private var isPresentingAdd = false
    @State private var newStationItem: NewSeismicStation? = nil

    @State private var itemBeingEdited: SensorItem? = nil

    @State private var pendingInitialX: Double? = nil
    @State private var pendingInitialY: Double? = nil
    
    @State private var quakeGesture = QuakeGestureState()

    var body: some View {
        NavigationSplitView {
            List(items, selection: $selection) { item in
                Text(item.title)
                    .tag(item)
                    .contentShape(Rectangle())
                    .onTapGesture { selection = item }
                    .swipeActions {
                        Button(role: .destructive) {
                            modelContext.delete(item)
                        } label: {
                            Label("Delete", systemImage: "trash")
                        }
                        Button("Edit") {
                            itemBeingEdited = item
                        }
                        .tint(.blue)
                    }
            }
            .navigationTitle("Seismic Stations")
            .toolbar {
                ToolbarItem(placement: .primaryAction) {
                    Button {
                        Task {
                            // TODO: default coordinates will be a center of the map...
                            let coordinate = CLLocationCoordinate2D(latitude: 0.0, longitude: 0.0)
                            let newRandomName = await generateStationName(at: coordinate)
                            
                            newStationItem = NewSeismicStation(name: newRandomName, coordinate: coordinate)
                        }
                    } label: {
                        Label("Add", systemImage: "plus")
                    }
                    .accessibilityLabel("Add a Seismic Station")
                }
                ToolbarItem(placement: .primaryAction) {
                    Button {
                        if let selected = selection {
                            itemBeingEdited = selected
                        }
                    } label: {
                        Label("Edit", systemImage: "pencil")
                    }
                    .disabled(selection == nil)
                    .accessibilityLabel("Edit selected Seismic Station")
                }
            }
            .sheet(
                item: $newStationItem,
                onDismiss: {
                    newStationItem = nil
                }
            ) { item in
                EditStationView(
                    title: "Add New Seismic Station",
                    confirmLabel: "Add",
                    initialX: item.coordinate.longitude,
                    initialY: item.coordinate.latitude,
                    name: item.name
                ) { x, y in
                    let new = SensorItem(x: x, y: y)
                    modelContext.insert(new)
                    selection = new
                }
            }
            .sheet(item: $itemBeingEdited) { item in
                EditStationView(
                    title: "Edit Seismic Station",
                    confirmLabel: "Update",
                    initialX: item.coordinate.x,
                    initialY: item.coordinate.y,
                ) { x, y in
                    item.x = x
                    item.y = y
                }
            }
        } detail: {
            ZStack {
                MapView(
                    sensors: items,
                    selectedCoordinate: selection?.coordinate,
                    onAddAt: { x, y in
                        pendingInitialX = x
                        pendingInitialY = y
                        isPresentingAdd = true
                    },
                    onQuakeAt: { x, y in
                        // TODO: launch the eather quake simulation
                    }
                )
                .ignoresSafeArea()
                
                QuakeHUDView()
                    .ignoresSafeArea()
            }.environment(quakeGesture)
        }
    }
}
