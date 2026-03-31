import SwiftUI
import CoreLocation


struct EditStationView: View {
    @Environment(\.dismiss) private var dismiss

    static private let validLongitude: ClosedRange<Double> = -180.0 ... 180.0
    static private let validLatitude: ClosedRange<Double> = -90.0 ... 90.0
    static private let defaultCoordinateFormat: FloatingPointFormatStyle<Double> = .number
        .precision(.fractionLength(2...5))
        .grouping(.never)

    @Bindable var station: StationItem

    private let viewTitle: String
    private let confirmTitle: String
    private let onSave: (() -> Void)?
    
    @State private var name: String = ""
    @State private var longitude: CLLocationDegrees = 0.0
    @State private var latitude: CLLocationDegrees = 0.0

    private var canSave: Bool { !name.isEmpty }

    init(
        title: String,
        confirmLabel: String,
        station: StationItem,
        onSave: (() -> Void)? = nil
    ) {
        self.viewTitle = title
        self.confirmTitle = confirmLabel
        self.station = station
        self.onSave = onSave
    }

    var body: some View {
        NavigationView {
            Form {
                TextField("Seismic Station Name", text: $name)
                    .keyboardType(.default)
                    .textContentType(.nickname)
                    .accessibilityLabel("Enter a station's name")
                    .onAppear { name = station.name }

                Section(header: Text("Coordinates")) {
                    TextField("Longitude", value: $longitude, format: Self.defaultCoordinateFormat)
                        .keyboardType(.decimalPad)
                        .textContentType(.oneTimeCode)
                        .accessibilityLabel("Longitude coordinate")
                        .disableAutocorrection(true)
                        .onAppear { longitude = station.longitude }
                        .onChange(of: longitude) { _, v in
                            longitude = v.clamped(to: Self.validLongitude)
                        }

                    TextField("Latitude", value: $latitude, format: Self.defaultCoordinateFormat)
                        .keyboardType(.decimalPad)
                        .textContentType(.oneTimeCode)
                        .accessibilityLabel("Latitude coordinate")
                        .disableAutocorrection(true)
                        .onAppear { latitude = station.latitude }
                        .onChange(of: latitude) { _, v in
                            latitude = v.clamped(to: Self.validLatitude)
                        }
                }
            }
            .navigationTitle(viewTitle)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        dismiss()
                    }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button(confirmTitle) {
                        station.name = name
                        station.longitude = longitude
                        station.latitude = latitude
                        dismiss()
                        onSave?()
                    }
                    .disabled(!canSave)
                }
            }
        }
    }
}

private extension Double {
    func clamped(to range: ClosedRange<Self>) -> Self {
        min(max(self, range.lowerBound), range.upperBound)
    }
}

//extension FormatStyle where Self == FloatingPointFormatStyle<Double> {
//    static let coordinate: Self = .number
//            .precision(.fractionLength(2...5))
//            .grouping(.never)
//}
