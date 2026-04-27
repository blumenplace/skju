import SwiftUI

// MARK: - Design tokens

private extension Color {
    static let seismicAccent = Color(red: 0,        green: 229/255, blue: 255/255) // #00e5ff
    static let seismicWarn   = Color(red: 255/255,  green: 107/255, blue: 53/255)  // #ff6b35
    static let seismicOk     = Color(red: 57/255,   green: 255/255, blue: 20/255)  // #39ff14
    static let seismicMid    = Color(red: 255/255,  green: 215/255, blue: 0)       // #ffd700
    static let seismicPanel  = Color(red: 11/255,   green: 17/255,  blue: 25/255)  // #0b1119
    static let seismicBorder = Color(red: 26/255,   green: 42/255,  blue: 58/255)  // #1a2a3a
    static let seismicDim    = Color(red: 74/255,   green: 96/255,  blue: 112/255) // #4a6070
    static let seismicText   = Color(red: 200/255,  green: 218/255, blue: 232/255) // #c8dae8
}

// MARK: - Sensor readings model

struct SensorReadings {
    var accelX: Int32 = 0
    var accelY: Int32 = 0
    var accelZ: Int32 = 0
    var gyroX: Int32 = 0
    var gyroY: Int32 = 0
    var gyroZ: Int32 = 0
}

// MARK: - Station card state

enum StationCardState {
    case nominal, active, triggered

    var borderColor: Color {
        switch self {
        case .nominal:   return .seismicBorder
        case .active:    return .seismicAccent
        case .triggered: return .seismicWarn
        }
    }

    var badgeLabel: String {
        switch self {
        case .nominal:   return "NOMINAL"
        case .active:    return "ACTIVE"
        case .triggered: return "TRIGGERED"
        }
    }

    var badgeColor: Color {
        switch self {
        case .nominal:   return .seismicOk
        case .active:    return .seismicAccent
        case .triggered: return .seismicWarn
        }
    }
}

// MARK: - Sensor reading row

private struct SensorReadingRow: View {
    let label: String
    let value: Int32
    let maxValue: Int32
    let gradientColors: [Color]

    private var fraction: Double {
        guard maxValue > 0 else { return 0 }
        return min(1.0, Double(abs(value)) / Double(maxValue))
    }

    var body: some View {
        HStack(spacing: 6) {
            Text(label)
                .font(.system(size: 10, design: .monospaced))
                .foregroundStyle(Color.seismicDim)
                .frame(width: 70, alignment: .leading)

            GeometryReader { geo in
                ZStack(alignment: .leading) {
                    RoundedRectangle(cornerRadius: 2)
                        .fill(Color.white.opacity(0.06))

                    RoundedRectangle(cornerRadius: 2)
                        .fill(
                            LinearGradient(
                                colors: gradientColors,
                                startPoint: .leading,
                                endPoint: .trailing
                            )
                        )
                        .frame(width: max(2, geo.size.width * fraction))
                        .animation(.easeOut(duration: 0.15), value: fraction)
                }
            }
            .frame(height: 5)

            Text(String(format: "%.3f", Double(value) / Double(maxValue)))
                .font(.system(size: 10, design: .monospaced))
                .foregroundStyle(Color.seismicText)
                .frame(width: 50, alignment: .trailing)
        }
    }
}

// MARK: - Station sensor card

struct StationSensorCard: View {
    let name: String
    let readings: SensorReadings
    let cardState: StationCardState
    // Units per full-scale — tune to match real sensor range
    let maxAccel: Int32
    let maxGyro: Int32
    let pWaveArrival: Double?
    let sWaveArrival: Double?

    private let accelGradient: [Color] = [.seismicAccent, Color(red: 0, green: 128/255, blue: 1)]
    private let gyroGradient:  [Color] = [.seismicMid,   Color(red: 1, green: 140/255, blue: 0)]

    init(
        name: String,
        readings: SensorReadings = SensorReadings(),
        cardState: StationCardState = .nominal,
        maxAccel: Int32 = 2000,
        maxGyro: Int32 = 2000,
        pWaveArrival: Double? = nil,
        sWaveArrival: Double? = nil
    ) {
        self.name = name
        self.readings = readings
        self.cardState = cardState
        self.maxAccel = maxAccel
        self.maxGyro = maxGyro
        self.pWaveArrival = pWaveArrival
        self.sWaveArrival = sWaveArrival
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            stationHeader
            accelRows
            gyroRows
            if pWaveArrival != nil || sWaveArrival != nil {
                arrivalTimes
            }
        }
        .padding(14)
        .background(Color.seismicPanel)
        .clipShape(RoundedRectangle(cornerRadius: 4))
        .overlay(
            RoundedRectangle(cornerRadius: 4)
                .stroke(cardState.borderColor, lineWidth: 1)
        )
        .animation(.easeInOut(duration: 0.2), value: cardState.borderColor)
    }

    private var stationHeader: some View {
        HStack {
            Text(name)
                .font(.system(size: 13, design: .monospaced))
                .foregroundStyle(Color.seismicAccent)
            Spacer()
            Text(cardState.badgeLabel)
                .font(.system(size: 9, design: .monospaced))
                .foregroundStyle(cardState.badgeColor)
                .padding(.horizontal, 7)
                .padding(.vertical, 2)
                .background(cardState.badgeColor.opacity(0.15))
                .clipShape(RoundedRectangle(cornerRadius: 2))
                .overlay(
                    RoundedRectangle(cornerRadius: 2)
                        .stroke(cardState.badgeColor, lineWidth: 1)
                )
        }
        .padding(.bottom, 4)
    }

    private var accelRows: some View {
        Group {
            SensorReadingRow(label: "ACCEL-X", value: readings.accelX, maxValue: maxAccel, gradientColors: accelGradient)
            SensorReadingRow(label: "ACCEL-Y", value: readings.accelY, maxValue: maxAccel, gradientColors: accelGradient)
            SensorReadingRow(label: "ACCEL-Z", value: readings.accelZ, maxValue: maxAccel, gradientColors: accelGradient)
        }
    }

    private var gyroRows: some View {
        Group {
            SensorReadingRow(label: "GYRO-X", value: readings.gyroX, maxValue: maxGyro, gradientColors: gyroGradient)
            SensorReadingRow(label: "GYRO-Y", value: readings.gyroY, maxValue: maxGyro, gradientColors: gyroGradient)
        }
    }

    private var arrivalTimes: some View {
            VStack(spacing: 6) {
                Divider()
                    .background(Color.seismicBorder)
                HStack {
                    if let p = pWaveArrival {
                        VStack(alignment: .leading, spacing: 2) {
                            Text("P-wave")
                                .font(.system(size: 10, design: .monospaced))
                                .foregroundStyle(Color.seismicDim)
                            Text(String(format: "%.2fs", p))
                                .font(.system(size: 10, design: .monospaced))
                                .foregroundStyle(Color.seismicOk)
                        }
                    }
                    Spacer()
                    if let s = sWaveArrival {
                        VStack(alignment: .leading, spacing: 2) {
                            Text("S-wave")
                                .font(.system(size: 10, design: .monospaced))
                                .foregroundStyle(Color.seismicDim)
                            Text(String(format: "%.2fs", s))
                                .font(.system(size: 10, design: .monospaced))
                                .foregroundStyle(Color.seismicWarn)
                        }
                    }
                }
            }
        }
    }

    // MARK: - Preview

    #Preview("Sensor Cards") {
        ScrollView {
            VStack(spacing: 12) {
                StationSensorCard(
                    name: "ST-1 / ALPHA",
                    readings: SensorReadings(accelX: 400, accelY: 300, accelZ: 200, gyroX: 160, gyroY: 240),
                    cardState: .active
                )
                StationSensorCard(
                    name: "ST-2 / BETA",
                    readings: SensorReadings(accelX: 1200, accelY: 900, accelZ: 600, gyroX: 500, gyroY: 700),
                    cardState: .triggered,
                    pWaveArrival: 3.14,
                    sWaveArrival: 5.39
                )
                StationSensorCard(
                    name: "ST-3 / GAMMA",
                    cardState: .nominal
                )
            }
            .padding()
        }
        .background(Color(red: 6/255, green: 10/255, blue: 15/255))
    }
