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
