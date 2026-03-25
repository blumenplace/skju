import SwiftUI


struct AddSensorView: View {
    @Environment(\.dismiss) private var dismiss

    @State private var xText: String = ""
    @State private var yText: String = ""

    private let viewTitle: String
    private let confirmTitle: String

    var onSave: (Double, Double) -> Void

    private var parsedX: Double? { Double(xText.trimmingCharacters(in: .whitespacesAndNewlines)) }
    private var parsedY: Double? { Double(yText.trimmingCharacters(in: .whitespacesAndNewlines)) }
    private var canSave: Bool { parsedX != nil && parsedY != nil }

    init(
        initialX: Double? = nil,
        initialY: Double? = nil,
        title: String = "Add Sensor",
        confirmLabel: String = "Save",
        onSave: @escaping (Double, Double) -> Void
    ) {
        let defaultPointFormat = "%.5f"

        _xText = State(initialValue: initialX.map { String(format: defaultPointFormat, $0) } ?? "")
        _yText = State(initialValue: initialY.map { String(format: defaultPointFormat, $0) } ?? "")
        self.viewTitle = title
        self.confirmTitle = confirmLabel
        self.onSave = onSave
    }

    var body: some View {
        NavigationView {
            Form {
                Section(header: Text("Coordinates")) {
                  TextField("X", text: $xText)
                    .keyboardType(.numbersAndPunctuation)
                    .textContentType(.oneTimeCode)
                    .accessibilityLabel("X coordinate")
                  TextField("Y", text: $yText)
                    .keyboardType(.numbersAndPunctuation)
                    .textContentType(.oneTimeCode)
                    .accessibilityLabel("Y coordinate")
                }
            }
            .navigationTitle(viewTitle)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button(confirmTitle) {
                        if let x = parsedX, let y = parsedY {
                            onSave(x, y)
                            dismiss()
                        }
                    }
                    .disabled(!canSave)
                }
            }
        }
    }
}
