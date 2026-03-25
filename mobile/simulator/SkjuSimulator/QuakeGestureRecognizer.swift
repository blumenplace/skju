import UIKit
import OSLog

final class QuakeGestureRecognizer: UIGestureRecognizer {

    private let doubleTapInterval: TimeInterval = 0.5
    
    private enum InternalGestureState {
        case waiting
        case firstTap(Date)
        case secondTap(startTime: Date, initialLocation: CGPoint, currentLocation: CGPoint?)
    }
    
    private var internalState: InternalGestureState = .waiting
    
    private var doubleTapTimer: Timer?
    
    private(set) var dragDistance: Double = 0
    
    private func calculateDistance(from start: CGPoint, to end: CGPoint) -> Double {
        let dx = end.x - start.x
        let dy = end.y - start.y
        return sqrt(dx * dx + dy * dy)
    }
    
    override func reset() {
        super.reset()
        internalState = .waiting
        dragDistance = 0
        doubleTapTimer?.invalidate()
        doubleTapTimer = nil
    }
    
    override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent) {
        super.touchesBegan(touches, with: event)
        
        Logger.ui.debug("touchesBegan")
        
        guard touches.count == 1, let touch = touches.first, let view = view else {
            state = .failed
            return
        }
        
        let location = touch.location(in: view)
        
        switch internalState {
        case .waiting:
            Logger.ui.debug("touchesBegan: waiting -> firstTap")
            internalState = .firstTap(Date())
            doubleTapTimer = Timer.scheduledTimer(withTimeInterval: doubleTapInterval, repeats: false) { [weak self] _ in
                self?.reset()
                self?.state = .failed
            }

            if state == .possible {
                state = .began
            } else if state == .began || state == .changed {
                state = .changed
            }
            
        case .firstTap(let firstTapTime), .secondTap(let firstTapTime, _, _):
            Logger.ui.debug("touchesBegan: -> secondTap")
            if Date().timeIntervalSince(firstTapTime) <= doubleTapInterval {
                doubleTapTimer?.invalidate()
                doubleTapTimer = nil
                internalState = .secondTap(startTime: Date(), initialLocation: location, currentLocation: nil)
                
                if state == .possible {
                    state = .began
                } else if state == .began || state == .changed {
                    state = .changed
                }
                
            } else {
                reset()
                state = .failed
            }
        }
    }
    
    override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent) {
        super.touchesMoved(touches, with: event)
        
        Logger.ui.debug("touchesMoved")
        
        guard let touch = touches.first, let view = view else {
            state = .failed
            return
        }
        
        let location = touch.location(in: view)
        
        if case let .secondTap(startTime, initialLoc, _) = internalState {
            internalState = .secondTap(startTime: startTime, initialLocation: initialLoc, currentLocation: location)
            
            if state == .possible {
                state = .began
            } else if state == .began || state == .changed {
                state = .changed
            }
        }
    }
    
    override func touchesEnded(_ touches: Set<UITouch>, with event: UIEvent) {
        super.touchesEnded(touches, with: event)
        
        Logger.ui.debug("touchesEnded")
        
        guard let touch = touches.first, let view = view else {
            state = .failed
            return
        }
        
        let location = touch.location(in: view)
        
        switch internalState {
        case .waiting:
            Logger.ui.debug("touchesEnded: waiting reset")
            reset()
            state = .failed

        case .firstTap:
            Logger.ui.debug("touchesEnded: firstTap do nothing")
            // do nothing, waiting for the second tap
            break
            
        case .secondTap(_, let initialLoc, let currentLoc):
            Logger.ui.debug("touchesEnded: secondTap")

            doubleTapTimer?.invalidate()
            doubleTapTimer = nil
            
            let finalLocation = currentLoc ?? location
            dragDistance = calculateDistance(from: initialLoc, to: finalLocation)
            
            state = .ended
        }
    }
    
    override func touchesCancelled(_ touches: Set<UITouch>, with event: UIEvent) {
        super.touchesCancelled(touches, with: event)
        
        Logger.ui.debug("touchesCancelled")
        
        internalState = .waiting
        doubleTapTimer?.invalidate()
        doubleTapTimer = nil
        state = .cancelled
    }
}
