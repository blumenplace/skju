import SwiftUI
import CoreLocation


struct EditStationView: View {
    @Environment(\.dismiss) private var dismiss
    
    static private let validLongitude: ClosedRange<Double> = -180.0 ... 180.0
    static private let validLatitude: ClosedRange<Double> = -90.0 ... 90.0

    static private let defaultPointFormat = "%.5f"

    @State private var name: String = ""
    @State private var latitude: CLLocationDegrees = 0.0
    @State private var longitude: CLLocationDegrees = 0.0

    private let viewTitle: String
    private let confirmTitle: String

    private var canSave: Bool { !name.isEmpty }

    var onSave: (Double, Double) -> Void

    init(
        title: String,
        confirmLabel: String,
        initialX: Double? = nil,
        initialY: Double? = nil,
        name: String = "",
        onSave: @escaping (Double, Double) -> Void
    ) {
        self.viewTitle = title
        self.confirmTitle = confirmLabel
        self.name = name

        self._longitude = State(initialValue: initialX ?? 0.0)
        self._latitude = State(initialValue: initialY ?? 0.0)

        self.onSave = onSave
    }

    var body: some View {
        NavigationView {
            Form {
                TextField("Seismic Station Name", text: $name)
                    .keyboardType(.default)
                    .textContentType(.nickname)
                    .accessibilityLabel("Enter a station's name")

                Section(header: Text("Coordinates")) {
                    TextField("Longitude", value: $longitude, format: .number)
                        .keyboardType(.decimalPad)
                        .textContentType(.oneTimeCode)
                        .accessibilityLabel("Longitude coordinate")
                        .disableAutocorrection(true)
                        .onChange(of: longitude) { _, v in
                            longitude = v.clamped(to: Self.validLongitude)
                        }

                    TextField("Latitude", value: $latitude, format: .number)
                        .keyboardType(.decimalPad)
                        .textContentType(.oneTimeCode)
                        .accessibilityLabel("Latitude coordinate")
                        .disableAutocorrection(true)
                        .onChange(of: latitude) { _, v in
                            latitude = v.clamped(to: Self.validLatitude)
                        }
                }
            }
            .navigationTitle(viewTitle)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button(confirmTitle) {
                        /*if let lon = parseCoordinate(longitude),
                           let lat = parseCoordinate(latitude)
                        {
                            onSave(lon, lat)
                            dismiss()
                        }*/
                        dismiss()
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
