import SwiftUI


struct EditStationView: View {
    @Environment(\.dismiss) private var dismiss

    static private let defaultPointFormat = "%.5f"

    @State private var name: String = ""
    @State private var longitude: String = ""
    @State private var latitude: String = ""

    private let viewTitle: String
    private let confirmTitle: String

    private var canSave: Bool {
        !name.isEmpty &&
            parseCoordinate(longitude) != nil &&
            parseCoordinate(latitude) != nil
    }

    var onSave: (Double, Double) -> Void

    init(
        title: String,
        confirmLabel: String,
        initialX: Double? = nil,
        initialY: Double? = nil,
        onSave: @escaping (Double, Double) -> Void
    ) {
        self.viewTitle = title
        self.confirmTitle = confirmLabel
        
        self._longitude = State(initialValue: initialX.map { String(format: Self.defaultPointFormat, $0) } ?? "")
        self._latitude = State(initialValue: initialY.map { String(format: Self.defaultPointFormat, $0) } ?? "")
        
        self.onSave = onSave
    }

    private func parseCoordinate(_ value: String) -> Double? {
        Double(value.trimmingCharacters(in: .whitespacesAndNewlines))
    }

    var body: some View {
        NavigationView {
            Form {
                TextField("Seismic Station Name", text: $name)
                    .keyboardType(.default)
                    .textContentType(.nickname)
                    .accessibilityLabel("Enter a station's name")

                Section(header: Text("Coordinates")) {
                    TextField("Longitude", text: $longitude)
                        .keyboardType(.numbersAndPunctuation)
                        .textContentType(.location)
                        .accessibilityLabel("Longitude coordinate")

                    TextField("Latitude", text: $latitude)
                        .keyboardType(.numbersAndPunctuation)
                        .textContentType(.location)
                        .accessibilityLabel("Latitude coordinate")
                }
            }
            .navigationTitle(viewTitle)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button(confirmTitle) {
                        if let lon = parseCoordinate(longitude),
                           let lat = parseCoordinate(latitude)
                        {
                            onSave(lon, lat)
                            dismiss()
                        }
                    }
                    .disabled(!canSave)
                }
            }
        }
    }
}
