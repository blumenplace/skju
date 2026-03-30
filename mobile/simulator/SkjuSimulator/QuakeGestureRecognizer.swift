import OSLog
import UIKit
import CoreLocation

protocol QuakeGestureRecognizerDelegate : AnyObject {
    func onQuakeGestureUpdate(phase: UIGestureRecognizer.State, origin: CGPoint, current: CGPoint)
}

final class QuakeGestureRecognizer: UILongPressGestureRecognizer {

    private var origin: CGPoint = .zero

    private var quakeDelegate: (any QuakeGestureRecognizerDelegate)?
    
    private override init(target: Any?, action: Selector?) {
        super.init(target: target, action: action)
    }

    init(target: any QuakeGestureRecognizerDelegate) {
        super.init(target: nil, action: nil)
        quakeDelegate = target
        
        minimumPressDuration = 0.5
        allowableMovement    = 100
        
        addTarget(self, action: #selector(handleGesture(_:)))
    }

    @objc private func handleGesture(_ recognizer: UILongPressGestureRecognizer) {
        guard let view else { return }

        let current = recognizer.location(in: view)

        switch recognizer.state {
        case .began:
            origin = current
            quakeDelegate?.onQuakeGestureUpdate(phase: .began, origin: origin, current: current)

        case .changed:
            quakeDelegate?.onQuakeGestureUpdate(phase: .changed, origin: origin, current: current)

        case .ended:
            quakeDelegate?.onQuakeGestureUpdate(phase: .ended, origin: origin, current: current)

        case .cancelled, .failed:
            quakeDelegate?.onQuakeGestureUpdate(phase: .cancelled, origin: origin, current: current)

        default:
            break
        }
    }

    /// Call from the `onUpdate` handler when the gesture activates (.began)
    /// and whenever the integer part of the magnitude crosses a threshold.
    static func impactFeedback(style: UIImpactFeedbackGenerator.FeedbackStyle = .light) {
        let generator = UIImpactFeedbackGenerator(style: style)
        generator.prepare()
        generator.impactOccurred()
    }

    /// Heavier feedback for the fire event.
    static func notificationFeedback(type: UINotificationFeedbackGenerator.FeedbackType = .success) {
        let generator = UINotificationFeedbackGenerator()
        generator.prepare()
        generator.notificationOccurred(type)
    }
}

extension CGPoint {
    var asClLocationCoordinate2D: CLLocationCoordinate2D {
        return CLLocationCoordinate2D(latitude: self.y, longitude: self.x)
    }
}
