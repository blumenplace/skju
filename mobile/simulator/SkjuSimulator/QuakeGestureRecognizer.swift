import UIKit

protocol QuakeGestureRecognizerDelegate : AnyObject {
    //
}

/// A continuous gesture recognizer that drives the quake simulation UI.
///
/// Lifecycle
/// ---------
/// 1. User presses and holds  →  `.began` fires after `minimumPressDuration`.
/// 2. User drags              →  `.changed` fires continuously.
///    - Horizontal delta  →  magnitude adjustment from default (3.5 M).
///    - Vertical delta    →  depth adjustment from default (100 km).
/// 3. User lifts finger       →  `.ended` fires; caller triggers simulation.
/// 4. System interrupts       →  `.cancelled` fires; caller resets UI.
///
/// The recognizer does **no** parameter computation itself — it simply
/// tracks the origin and current touch point and publishes them via
/// `onUpdate`. All math lives in `QuakeParameters`.
final class QuakeGestureRecognizer: UILongPressGestureRecognizer {

    /// Maximum finger movement (pts) allowed during the press phase
    /// before the gesture fails. Keep generous so a shaky hold still arms.
    // allowableMovement is already declared on the superclass.

    /// Called on every meaningful state change.
    ///
    /// Parameters:
    /// - `phase`   : current recognizer phase (maps to UIGestureRecognizer.State)
    /// - `origin`  : point where the long press was first detected (view coords)
    /// - `current` : current touch point (view coords); equals origin on `.began`
    var onUpdate: ((_ phase: UIGestureRecognizer.State,
                    _ origin: CGPoint,
                    _ current: CGPoint) -> Void)?

    private var origin: CGPoint = .zero

    override init(target: Any?, action: Selector?) {
        super.init(target: target, action: action)
        minimumPressDuration = 0.48
        allowableMovement    = 20
        addTarget(self, action: #selector(handleGesture(_:)))
    }

    @objc private func handleGesture(_ recognizer: UILongPressGestureRecognizer) {
        guard let view else { return }

        let current = recognizer.location(in: view)

        switch recognizer.state {
        case .began:
            origin = current
            onUpdate?(.began, origin, current)

        case .changed:
            onUpdate?(.changed, origin, current)

        case .ended:
            onUpdate?(.ended, origin, current)

        case .cancelled, .failed:
            onUpdate?(.cancelled, origin, current)

        default:
            break
        }
    }

    // MARK: - Haptic feedback

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


/*
import UIKit
import OSLog

protocol QuakeGestureRecognizerDelegate: AnyObject {
    func quakeGestureDidBegin(location: CGPoint)
    func quakeMagnitudeDidChange(location: CGPoint, magnitude: Double)
    func quakeGestureDidEnd(location: CGPoint, magnitude: Double)
    func quakeGestureDidCancel()
}

final class QuakeGestureRecognizer: UIGestureRecognizer {

    private let doubleTapInterval: TimeInterval = 1.0
    
    private enum InternalGestureState {
        case waiting
        case waitingLongTap(timer: Timer, coordinate: CGPoint)
        case longTap(coordinate: CGPoint)
    }

    private var internalState: InternalGestureState = .waiting

    weak var quakeDelegate: QuakeGestureRecognizerDelegate? = nil

    private func calculateMagnitude(from start: CGPoint, to end: CGPoint) -> Double {
        let dx = end.x - start.x
        let dy = end.y - start.y
        return sqrt(dx * dx + dy * dy)
    }

    override func reset() {
        super.reset()
        resetInternalState()
    }

    private final func resetInternalState() {
        if case .waitingLongTap(let timer, _) = internalState {
            timer.invalidate()
        }
        internalState = .waiting
    }

    override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent) {
        super.touchesBegan(touches, with: event)
        
        guard touches.count == 1, let touch = touches.first, let view = view else {
            state = .failed
            return
        }
        
        let location = touch.location(in: view)
        
        switch internalState {
        case .waiting:
            let timer = Timer.scheduledTimer(withTimeInterval: doubleTapInterval, repeats: false) { [weak self] _ in
                if let s = self {
                    if s.state == .began {
                        s.internalState = .longTap(coordinate: location)
                        s.state = .began
                    } else {
                        s.state = .failed
                    }
                }
            }

            if state == .possible {
                internalState = .waitingLongTap(timer: timer, coordinate: location)
                state = .began
            } else {
                state = .failed
            }
            
        case .waitingLongTap(timer: let timer, _):
            timer.invalidate()
            state = .failed
            
        case .longTap:
            state = .failed
        }
    }
    
    override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent) {
        super.touchesMoved(touches, with: event)

        guard let touch = touches.first, let view = view else {
            state = .failed
            return
        }

        let location = touch.location(in: view)
        
        if case .longTap(let coordinate) = internalState {
            if state == .began || state == .changed {
                let currentMagnitude = calculateMagnitude(from: coordinate, to: location)
                internalState = .longTap(coordinate: coordinate)
                state = .changed

                quakeDelegate?.quakeMagnitudeDidChange(location: coordinate, magnitude: currentMagnitude)
            } else {
                state = .failed
            }
        }
    }
    
    override func touchesEnded(_ touches: Set<UITouch>, with event: UIEvent) {
        super.touchesEnded(touches, with: event)
        
        guard let touch = touches.first, let view = view else {
            state = .failed
            return
        }
        
        let location = touch.location(in: view)
        
        switch internalState {
        case .waiting:
            state = .failed

        case .waitingLongTap(let timer, _):
            timer.invalidate()
            state = .failed

        case .longTap(let coordinate):
            if state == .began || state == .changed {
                let currentMagnitude = calculateMagnitude(from: coordinate, to: location)
                resetInternalState()
                state = .ended
                
                quakeDelegate?.quakeGestureDidEnd(location: coordinate, magnitude: currentMagnitude)
            } else {
                state = .failed
            }
        }
    }
    
    override func touchesCancelled(_ touches: Set<UITouch>, with event: UIEvent) {
        super.touchesCancelled(touches, with: event)
        
        quakeDelegate?.quakeGestureDidCancel()
        resetInternalState()
        state = .cancelled
    }
}
*/
