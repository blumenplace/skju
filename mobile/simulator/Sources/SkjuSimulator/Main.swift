import MapKit
import OSLog
import SwiftData
import SwiftUI
import UIKit

extension Logger {
  private static var subsystem = Bundle.main.bundleIdentifier!

  static let network = Logger(subsystem: subsystem, category: "Network")
  static let ui = Logger(subsystem: subsystem, category: "UI")
}

@main
struct OpenStreetMapApp: App {
  var body: some Scene {
    WindowGroup {
      ContentView()
    }
    .modelContainer(
      for: SensorItem.self,
      /*configurations: [
        // Use our custom remote SwiftData store (iOS 18+)
        SensorStoreConfiguration(
          identifier: "SensorRemoteStore",
          baseURL: URL(string: "https://skju-sim.blumen.place")!
        )
      ]*/
    )
  }
}

public struct Coordinate: Hashable {
  let x: Double
  let y: Double
}

@Model final public class SensorItem: Identifiable, Hashable {
  public var id: UUID
  var x: Double
  var y: Double

  init(id: UUID = UUID(), x: Double, y: Double) {
    self.id = id
    self.x = x
    self.y = y
  }

  var coordinate: Coordinate { Coordinate(x: x, y: y) }
  var title: String { "x: \(x), y: \(y)" }

  public static func == (lhs: SensorItem, rhs: SensorItem) -> Bool { lhs.id == rhs.id }
  public func hash(into hasher: inout Hasher) { hasher.combine(id) }
}

struct ContentView: View {
  @Environment(\.modelContext) private var modelContext
  @Query(sort: [SortDescriptor(\SensorItem.x), SortDescriptor(\SensorItem.y)]) private var items:
    [SensorItem]

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
          .onTapGesture {
            selection = item
          }
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

  init(
    initialX: Double? = nil,
    initialY: Double? = nil,
    title: String = "Add Sensor",
    confirmLabel: String = "Save",
    onSave: @escaping (Double, Double) -> Void
  ) {
    let defaultPointFormat = "%.5f"

    _xText = State(initialValue: initialX.map { String(format: defaultPointFormat, $0) } ?? "")
    _yText = State(initialValue: initialY.map { String(format: defaultPointFormat, $0) } ?? "")
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
            .textContentType(.oneTimeCode)
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
  var sensors: [SensorItem] = []
  var selectedCoordinate: Coordinate? = nil
  var onAddAt: ((Double, Double) -> Void)? = nil
  var onQuakeAt: ((Double, Double) -> Void)? = nil

  func makeUIView(context: Context) -> MKMapView {
    let mapView = MKMapView()

    // Configure OpenStreetMap Tile Overlay
    let template = "https://tile.openstreetmap.org/{z}/{x}/{y}.png"
    let overlay = MKTileOverlay(urlTemplate: template)
    overlay.canReplaceMapContent = true
    mapView.addOverlay(overlay, level: .aboveLabels)
    mapView.delegate = context.coordinator

    // Add context menu interaction to show a popup near the press location
    let interaction = UIContextMenuInteraction(delegate: context.coordinator)
    mapView.addInteraction(interaction)

    return mapView
  }

  func updateUIView(_ uiView: MKMapView, context: Context) {
    if let sel = selectedCoordinate {
      let center = CLLocationCoordinate2D(latitude: sel.y, longitude: sel.x)

      let span = MKCoordinateSpan(latitudeDelta: 1.0, longitudeDelta: 1.0)
      let region = MKCoordinateRegion(center: center, span: span)
      uiView.setRegion(region, animated: true)
    }

    let existing = uiView.annotations
    uiView.removeAnnotations(existing)

    for sensor in sensors {
      let ann = MKPointAnnotation()
      ann.coordinate = CLLocationCoordinate2D(
        latitude: sensor.coordinate.y, longitude: sensor.coordinate.x)
      ann.title = "Sensor"
      uiView.addAnnotation(ann)
    }
  }

  func makeCoordinator() -> Coordinator {
    Coordinator(self)
  }

  class Coordinator: NSObject, MKMapViewDelegate, UIContextMenuInteractionDelegate {
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

    func mapView(_ mapView: MKMapView, viewFor annotation: MKAnnotation) -> MKAnnotationView? {
      // Customize only for our point annotations
      guard annotation is MKPointAnnotation else { return nil }
      let identifier = "sensor-annotation"
      let view: MKMarkerAnnotationView
      if let dequeued = mapView.dequeueReusableAnnotationView(withIdentifier: identifier)
        as? MKMarkerAnnotationView
      {
        view = dequeued
        view.annotation = annotation
      } else {
        view = MKMarkerAnnotationView(annotation: annotation, reuseIdentifier: identifier)
        view.canShowCallout = true
        view.glyphImage = UIImage(systemName: "flag.fill")
        view.markerTintColor = .systemRed
      }
      return view
    }

    // UIContextMenuInteractionDelegate
    func contextMenuInteraction(
      _ interaction: UIContextMenuInteraction, configurationForMenuAtLocation location: CGPoint
    ) -> UIContextMenuConfiguration? {
      guard let mapView = interaction.view as? MKMapView else { return nil }
      let coord = mapView.convert(location, toCoordinateFrom: mapView)
      let x = coord.longitude
      let y = coord.latitude
      return UIContextMenuConfiguration(identifier: nil, previewProvider: nil) { _ in
        let add = UIAction(title: "Add Sensor", image: UIImage(systemName: "plus")) {
          [weak self] _ in
          self?.parent.onAddAt?(x, y)
        }
        let quake = UIAction(title: "Quake", image: UIImage(systemName: "waveform.path.ecg")) {
          [weak self] _ in
          self?.parent.onQuakeAt?(x, y)
        }
        return UIMenu(title: "Map", children: [add, quake])
      }
    }
  }
}
