import Foundation
import SwiftData

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
