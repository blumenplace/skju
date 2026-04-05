import SwiftUI

struct ColorIntensityBar: View {
    let intensity: Double
    let isActive: Bool
 
    private var barColor: Color {
        if intensity < 0 {
            return Color(
                red: 1.0,
                green: max(0, 1.0 + intensity),
                blue: max(0, 1.0 + intensity)
            )
        } else {
            return Color(
                red: max(0, 1.0 - intensity),
                green: 1.0,
                blue: max(0, 1.0 - intensity)
            )
        }
    }
 
    private var label: String {
        if abs(intensity) < 0.05 { return "Hold & Drag" }
        return intensity < 0 ? "↑ Dragging Up" : "↓ Dragging Down"
    }
 
    var body: some View {
        VStack(spacing: 0) {
            Text(label)
                .font(.system(.headline, design: .rounded))
                .foregroundColor(.white)
                .padding(.vertical, 12)
                .frame(maxWidth: .infinity)
                .background(Color.black.opacity(0.3))
 
            GeometryReader { geo in
                ZStack(alignment: intensity < 0 ? .top : .bottom) {
                    // Background track
                    RoundedRectangle(cornerRadius: 8)
                        .fill(Color.white.opacity(0.15))
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
 
                    // Filled portion
                    RoundedRectangle(cornerRadius: 8)
                        .fill(
                            LinearGradient(
                                colors: [barColor.opacity(0.6), barColor],
                                startPoint: intensity < 0 ? .bottom : .top,
                                endPoint: intensity < 0 ? .top : .bottom
                            )
                        )
                        .frame(
                            maxWidth: .infinity,
                            maxHeight: geo.size.height * abs(intensity)
                        )
                }
            }
            .padding(.horizontal, 20)
            .padding(.vertical, 12)
 
            Text(String(format: "%+.0f%%", intensity * 100))
                .font(.system(.title2, design: .monospaced).bold())
                .foregroundColor(barColor)
                .padding(.bottom, 16)
        }
        .frame(width: 160)
        .background(
            RoundedRectangle(cornerRadius: 16)
                .fill(Color.black.opacity(0.5))
                .shadow(color: barColor.opacity(isActive ? 0.7 : 0.2), radius: isActive ? 20 : 6)
        )
        .overlay(
            RoundedRectangle(cornerRadius: 16)
                .stroke(barColor.opacity(isActive ? 0.9 : 0.3), lineWidth: isActive ? 2 : 1)
        )
        .animation(.spring(response: 0.3, dampingFraction: 0.7), value: intensity)
        .animation(.easeInOut(duration: 0.2), value: isActive)
    }
}
