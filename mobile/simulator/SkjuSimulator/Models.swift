import Foundation
import SwiftData
import CoreLocation

@Model final public class StationItem: Identifiable, Hashable {
    public var id: UUID

    var name: String
    var latitude: CLLocationDegrees
    var longitude: CLLocationDegrees

    var coordinate: CLLocationCoordinate2D {
        CLLocationCoordinate2D(latitude: latitude, longitude: longitude)
    }

    init(id: UUID, name: String, lat: CLLocationDegrees, lon: CLLocationDegrees) {
        self.id = id
        self.name = name
        self.latitude = lat
        self.longitude = lon
    }

    convenience init(id: UUID, name: String, coordinate: CLLocationCoordinate2D) {
        self.init(id: id, name: name, lat: coordinate.latitude, lon: coordinate.longitude)
    }

    public static func == (lhs: StationItem, rhs: StationItem) -> Bool { lhs.id == rhs.id }

    public func hash(into hasher: inout Hasher) { hasher.combine(id) }
}

/// Event structure is defined in "eventspod.h" file.
extension Event {
    init(sensor_id: UInt64, date: Date) {
        let ts = UInt64(date.timeIntervalSince1970)
        self = Event(
            sensor_id: sensor_id,
            ts: ts,
            gyro_x: 0,
            gyro_y: 0,
            gyro_z: 0,
            accel_x: 0,
            accel_y: 0,
            accel_z: 0
        )
    }
}
