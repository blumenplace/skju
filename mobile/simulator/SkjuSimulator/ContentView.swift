import MapKit
import SwiftData
import SwiftUI
import UIKit


struct ContentView: View {
    @Environment(\.modelContext) private var modelContext

    @Query(sort: [SortDescriptor(\StationItem.id)]) private var stations: [StationItem]
    @State private var selection: StationItem? = nil
    @State private var newStation: StationItem? = nil
    @State private var editedStation: StationItem? = nil
    
    @State private var quakeGesture = QuakeGestureState()

    var body: some View {
        NavigationSplitView {
            List(stations, selection: $selection) { item in
                Text(item.name)
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
                            editedStation = item
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
                            let (id: id, name: name) = await generateStationName(at: coordinate)
                            newStation = StationItem(id: id, name: name, coordinate: coordinate)
                        }
                    } label: {
                        Label("Add", systemImage: "plus")
                    }
                    .accessibilityLabel("Add a Seismic Station")
                }
                ToolbarItem(placement: .primaryAction) {
                    Button {
                        if let selected = selection {
                            editedStation = selected
                        }
                    } label: {
                        Label("Edit", systemImage: "pencil")
                    }
                    .disabled(selection == nil)
                    .accessibilityLabel("Edit selected Seismic Station")
                }
            }
            .sheet(item: $newStation, onDismiss: { newStation = nil }) { item in
                EditStationView(
                    title: "Add New Seismic Station",
                    confirmLabel: "Add",
                    station: item
                ) {
                    modelContext.insert(item)
                    selection = item
                }
            }
            .sheet(item: $editedStation) { item in
                EditStationView(
                    title: "Edit Seismic Station",
                    confirmLabel: "Update",
                    station: item,
                ) {
                    selection = item
                }
            }
        } detail: {
            ZStack {
                MapView(
                    sensors: stations,
                    selectedCoordinate: selection?.coordinate,
                    onAddAt: { lon, lat in
                        Task {
                            let coordinate = CLLocationCoordinate2D(latitude: lat, longitude: lon)
                            let (id: id, name: name) = await generateStationName(at: coordinate)
                            newStation = StationItem(id: id, name: name, coordinate: coordinate)
                        }
                    },
                    onQuakeAt: { lon, lat in
                        // TODO: launch the eather quake simulation
                    }
                )
                .ignoresSafeArea()
                
                QuakeHUDView().ignoresSafeArea()
            }.environment(quakeGesture)
        }
    }
}
