import CoreLocation


struct QuakeParameters {

    static let defaultMagnitude: Double = 3.5
    static let defaultDepthKm:   Double = 100.0

    /// Points of horizontal drag per 1.0 magnitude unit.
    static let ptsPerMagnitudeUnit: CGFloat = 20.0

    /// Points of vertical drag per 1 km of depth.
    /// Dragging down increases depth; dragging up decreases it.
    static let ptsPerDepthKm: CGFloat = 0.22

    static let magnitudeRange: ClosedRange<Double> = 0.0 ... 9.5
    static let depthKmRange:   ClosedRange<Double> = 0.0 ... 700.0

    /// Richter / moment magnitude (0.0 – 9.5).
    let magnitude: Double

    /// Hypocentre depth in kilometres (0 – 700 km).
    let depthKm: Double

    /// Map coordinate of the epicentre.
    /// Populated once the gesture commits; `nil` while still dragging.
    let coordinate: CLLocationCoordinate2D?

    init(
        magnitude:  Double = defaultMagnitude,
        depthKm:    Double = defaultDepthKm,
        coordinate: CLLocationCoordinate2D? = nil
    ) {
        self.magnitude  = magnitude.clamped(to: Self.magnitudeRange)
        self.depthKm    = depthKm.clamped(to: Self.depthKmRange)
        self.coordinate = coordinate
    }

    /// Compute parameters from a raw gesture delta (in points)
    /// relative to the long-press origin.
    ///
    /// - Parameter delta: `CGPoint` where x = horizontal offset,
    ///   y = vertical offset (positive = down = deeper).
    static func from(delta: CGPoint, coordinate: CLLocationCoordinate2D? = nil) -> QuakeParameters {
        let mag = defaultMagnitude + Double(delta.x) / Double(ptsPerMagnitudeUnit)
        let dep = defaultDepthKm  + Double(delta.y) / Double(ptsPerDepthKm)
        return QuakeParameters(magnitude: mag, depthKm: dep, coordinate: coordinate)
    }

    /// Simplified Modified Mercalli Intensity at the epicentre.
    /// Returns a Roman numeral string (I – X) and a colour name.
    var epicentralIntensity: (roman: String, colorName: String) {
        guard magnitude >= 0.5 else { return ("—", "gray") }
        let rawIndex = magnitude / Self.magnitudeRange.upperBound * 10.0
        * (1.0 - depthKm / Self.depthKmRange.upperBound * 0.4)
        let index    = max(0, min(9, Int(rawIndex)))
        let romans   = ["I","II","III","IV","V","VI","VII","VIII","IX","X"]
        let colors   = ["systemGray","systemBlue","systemTeal",
                        "systemGreen","systemYellow","systemOrange",
                        "systemOrange","systemRed","systemRed","systemPurple"]
        return (romans[index], colors[index])
    }
}

private extension Comparable {
    func clamped(to range: ClosedRange<Self>) -> Self {
        min(max(self, range.lowerBound), range.upperBound)
    }
}
