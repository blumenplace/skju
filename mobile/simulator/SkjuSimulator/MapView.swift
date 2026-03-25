import SwiftUI
import MapKit

extension MKMapRect {
    init(for region: MKCoordinateRegion) {
        let topLeft = CLLocationCoordinate2D(
            latitude: region.center.latitude + region.span.latitudeDelta / 2,
            longitude: region.center.longitude - region.span.longitudeDelta / 2
        )
        let bottomRight = CLLocationCoordinate2D(
            latitude: region.center.latitude - region.span.latitudeDelta / 2,
            longitude: region.center.longitude + region.span.longitudeDelta / 2
        )
        let a = MKMapPoint(topLeft)
        let b = MKMapPoint(bottomRight)
        self = MKMapRect(
            x: min(a.x, b.x),
            y: min(a.y, b.y),
            width: abs(a.x - b.x),
            height: abs(a.y - b.y)
        )
    }
}

final class QuakeOverlay: NSObject, MKOverlay {
    let boundingMapRect: MKMapRect
    let coordinate: CLLocationCoordinate2D

    init(region: MKCoordinateRegion) {
        self.boundingMapRect = MKMapRect(for: region)
        self.coordinate = region.center
        super.init()
    }
}

final class QuakeRenderer: MKOverlayRenderer {
    override func draw(_ mapRect: MKMapRect, zoomScale: MKZoomScale, in context: CGContext) {
        // Convert a map rect to view rect
        let rect = self.rect(for: overlay.boundingMapRect)

        // Example Core Graphics drawing
        context.setFillColor(UIColor.systemRed.withAlphaComponent(0.15).cgColor)
        context.fill(rect)

        // Draw a stroked circle at a map coordinate
        let centerCoord = overlay.coordinate
        let centerPoint = self.point(for: MKMapPoint(centerCoord))
        let radius: CGFloat = 60 / zoomScale  // scale with zoom to keep visual size reasonable
        context.setStrokeColor(UIColor.systemRed.cgColor)
        context.setLineWidth(2 / zoomScale)
        context.strokeEllipse(
          in: CGRect(
            x: centerPoint.x - radius,
            y: centerPoint.y - radius,
            width: radius * 2, height: radius * 2)
        )
    }
}


struct MapView: UIViewRepresentable {
    var sensors: [SensorItem] = []
    var selectedCoordinate: Coordinate? = nil
    var onAddAt: ((Double, Double) -> Void)? = nil
    var onQuakeAt: ((Double, Double) -> Void)? = nil

    func makeUIView(context: Context) -> MKMapView {
        let mapView = MKMapView()

        let overlay = MKTileOverlay(urlTemplate: "https://tile.openstreetmap.org/{z}/{x}/{y}.png")
        overlay.canReplaceMapContent = true
        mapView.addOverlay(overlay, level: .aboveLabels)

        let quakeOverlay = QuakeOverlay(region: mapView.region)
        mapView.addOverlay(quakeOverlay, level: .aboveLabels)

        mapView.delegate = context.coordinator

        // Add context menu interaction to show a popup near the press location
        let interaction = UIContextMenuInteraction(delegate: context.coordinator)
        mapView.addInteraction(interaction)

        // Add force/long press drag gesture
        let quakeGesture = QuakeGestureRecognizer(target: context.coordinator, action: #selector(Coordinator.handleForceDrag(_:)))
        quakeGesture.delegate = context.coordinator
//        quakeGesture.cancelsTouchesInView = false
        mapView.addGestureRecognizer(quakeGesture)

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
    
    func onQuake(x: Double, y: Double, intensity: CGFloat) {
        print("QUAKE AT \(x), \(y) with intensity \(intensity)")
    }

    class Coordinator: NSObject, MKMapViewDelegate, UIContextMenuInteractionDelegate, UIGestureRecognizerDelegate {
        var parent: MapView

        init(_ parent: MapView) {
            self.parent = parent
        }

        @objc func handleForceDrag(_ gesture: QuakeGestureRecognizer) {
            guard let mapView = gesture.view as? MKMapView else { return }
            
            // Only trigger on .ended - when the user lifts their finger
            if gesture.state == .ended {
                let location = gesture.location(in: mapView)
                let coord = mapView.convert(location, toCoordinateFrom: mapView)
                let x = coord.longitude
                let y = coord.latitude
                
                let dragDistance = gesture.dragDistance
                
                parent.onQuake(x: x, y: y, intensity: dragDistance)
            }
        }
        
        // Allow simultaneous gestures so map interactions still work
        func gestureRecognizer(_ gestureRecognizer: UIGestureRecognizer, shouldRecognizeSimultaneouslyWith otherGestureRecognizer: UIGestureRecognizer) -> Bool {
            return false  // Set to true if you want map gestures to work simultaneously
        }

        func mapView(_ mapView: MKMapView, rendererFor overlay: MKOverlay) -> MKOverlayRenderer {
            if let tileOverlay = overlay as? MKTileOverlay {
                return MKTileOverlayRenderer(tileOverlay: tileOverlay)
            }
            return MKOverlayRenderer(overlay: overlay)
        }

        func mapView(_ mapView: MKMapView, viewFor annotation: MKAnnotation) -> MKAnnotationView? {
            guard annotation is MKPointAnnotation else { return nil }
            let identifier = "sensor-annotation"
            let view: MKMarkerAnnotationView
            if let dequeued = mapView.dequeueReusableAnnotationView(withIdentifier: identifier) as? MKMarkerAnnotationView {
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
                let quake = UIAction(title: "Quake", image: UIImage(systemName: "waveform.path.ecg")) { [weak self] _ in
                    self?.parent.onQuakeAt?(x, y)
                }
                return UIMenu(title: "Map", children: [add, quake])
            }
        }
    }
}
