import MapKit
import SwiftData
import SwiftUI
import UIKit


struct ContentView: View {
    @Environment(\.modelContext) private var modelContext
    @Query(sort: [SortDescriptor(\SensorItem.x), SortDescriptor(\SensorItem.y)]) private var items: [SensorItem]

    @State private var selection: SensorItem? = nil

    @State private var isPresentingAdd = false
    @State private var itemBeingEdited: SensorItem? = nil

    @State private var pendingInitialX: Double? = nil
    @State private var pendingInitialY: Double? = nil

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
            .navigationTitle("Sensors")
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
                        if let selected = selection {
                            itemBeingEdited = selected
                        }
                    } label: {
                        Label("Edit", systemImage: "pencil")
                    }
                    .disabled(selection == nil)
                    .accessibilityLabel("Edit selected sensor")
                }
            }
            .sheet(
                isPresented: $isPresentingAdd,
                onDismiss: {
                    pendingInitialX = nil
                    pendingInitialY = nil
                }
            ) {
                AddSensorView(
                    initialX: pendingInitialX,
                    initialY: pendingInitialY
                ) { x, y in
                    let new = SensorItem(x: x, y: y)
                    modelContext.insert(new)
                    selection = new
                }
            }
            .sheet(item: $itemBeingEdited) { item in
                AddSensorView(
                    initialX: item.coordinate.x,
                    initialY: item.coordinate.y,
                    title: "Edit Sensor",
                    confirmLabel: "Update"
                ) { x, y in
                    item.x = x
                    item.y = y
                }
            }
        } detail: {
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
        }
    }
}
