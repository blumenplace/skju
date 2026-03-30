import SwiftUI
import CoreLocation

/// Observable bridge between the UIKit gesture recognizer
/// and SwiftUI views. All properties are updated on the main actor.
@Observable
@MainActor
final class QuakeGestureState {
    var phase: QuakeGesturePhase = .idle

    /// Where the long press began — the crosshair origin.
    var originPoint: CGPoint = .zero

    /// Current finger position while dragging.
    var currentPoint: CGPoint = .zero

    /// Delta from origin to current finger position.
    var delta: CGPoint {
        CGPoint(
            x: currentPoint.x - originPoint.x,
            y: currentPoint.y - originPoint.y
        )
    }

    /// Parameters computed from the current drag delta.
    var parameters: QuakeParameters = QuakeParameters()

    /// Set when the user lifts their finger; consumed by the simulation engine.
    var lastFiredParameters: QuakeParameters? = nil

    func gestureArming(at point: CGPoint) {
        phase        = .arming
        originPoint  = point
        currentPoint = point
        parameters   = QuakeParameters()
    }

    func gestureActivated(at point: CGPoint) {
        phase        = .active
        originPoint  = point
        currentPoint = point
        parameters   = QuakeParameters()  // reset to defaults at activation
    }

    func gestureMoved(to point: CGPoint, coordinate: CLLocationCoordinate2D?) {
        guard phase == .active else { return }
        currentPoint = point
        parameters   = QuakeParameters.from(delta: delta, coordinate: coordinate)
    }

    func gestureFired(coordinate: CLLocationCoordinate2D?) {
        // Stamp the coordinate on the final parameters before firing
        let final = QuakeParameters(
            magnitude:  parameters.magnitude,
            depthKm:    parameters.depthKm,
            coordinate: coordinate
        )
        lastFiredParameters = final
        phase = .fired

        // Return to idle after a short beat so the map can
        // animate the wave rings before the HUD vanishes.
        Task {
            try? await Task.sleep(for: .milliseconds(300))
            phase = .idle
        }
    }

    func gestureCancelled() {
        phase = .idle
    }
}

enum QuakeGesturePhase: Equatable {
    case idle
    case arming
    case active
    case fired
}
