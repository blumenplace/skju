import SwiftUI


struct QuakeHUDView: View {

    @Environment(QuakeGestureState.self) var gesture

    var body: some View {
        ZStack {
            if gesture.phase == .active || gesture.phase == .fired {
                CrosshairView(origin: gesture.originPoint)
                IndicatorBarsView(
                    origin:  gesture.originPoint,
                    delta:   gesture.delta
                )
                FingerCircleView(position: gesture.currentPoint)
                ParameterCardView(
                    parameters:   gesture.parameters,
                    fingerPoint:  gesture.currentPoint
                )
            }
        }
        // The HUD must not swallow touches — the map beneath still needs them.
        .allowsHitTesting(false)
        // Fade in/out with the phase change.
        .animation(.easeInOut(duration: 0.15), value: gesture.phase)
    }
}

private struct CrosshairView: View {

    let origin: CGPoint

    private let armLength: CGFloat = 110
    private let tickLength: CGFloat = 8

    var body: some View {
        Canvas { context, _ in
            let lineColor = Color.white.opacity(0.2)
            let tickColor = Color.white.opacity(0.5)

            // Horizontal arm
            var hPath = Path()
            hPath.move(to: CGPoint(x: origin.x - armLength, y: origin.y))
            hPath.addLine(to: CGPoint(x: origin.x + armLength, y: origin.y))
            context.stroke(hPath, with: .color(lineColor), lineWidth: 1)

            // Vertical arm
            var vPath = Path()
            vPath.move(to: CGPoint(x: origin.x, y: origin.y - armLength))
            vPath.addLine(to: CGPoint(x: origin.x, y: origin.y + armLength))
            context.stroke(vPath, with: .color(lineColor), lineWidth: 1)

            // End ticks — horizontal
            for dx: CGFloat in [-armLength, armLength] {
                var t = Path()
                t.move(to:    CGPoint(x: origin.x + dx, y: origin.y - tickLength / 2))
                t.addLine(to: CGPoint(x: origin.x + dx, y: origin.y + tickLength / 2))
                context.stroke(t, with: .color(tickColor), lineWidth: 1)
            }

            // End ticks — vertical
            for dy: CGFloat in [-armLength, armLength] {
                var t = Path()
                t.move(to:    CGPoint(x: origin.x - tickLength / 2, y: origin.y + dy))
                t.addLine(to: CGPoint(x: origin.x + tickLength / 2, y: origin.y + dy))
                context.stroke(t, with: .color(tickColor), lineWidth: 1)
            }

            // Centre dot
            let dotRect = CGRect(x: origin.x - 4, y: origin.y - 4, width: 8, height: 8)
            context.fill(Path(ellipseIn: dotRect), with: .color(.white.opacity(0.8)))
        }
        // Axis labels
        .overlay(axisLabels)
    }

    @ViewBuilder
    private var axisLabels: some View {
        // Magnitude labels
        Text("← LOWER")
            .hudAxisLabel()
            .position(x: origin.x - armLength * 0.55, y: origin.y - 14)

        Text("HIGHER →")
            .hudAxisLabel()
            .position(x: origin.x + armLength * 0.55, y: origin.y - 14)

        // Depth labels
        Text("↑ SHALLOWER")
            .hudAxisLabel()
            .position(x: origin.x + 52, y: origin.y - armLength * 0.6)

        Text("↓ DEEPER")
            .hudAxisLabel()
            .position(x: origin.x + 44, y: origin.y + armLength * 0.6)
    }
}

private struct IndicatorBarsView: View {

    let origin: CGPoint
    let delta:  CGPoint

    var body: some View {
        Canvas { context, _ in
            drawMagBar(context)
            drawDepBar(context)
        }
    }

    private func drawMagBar(_ context: GraphicsContext) {
        let dx = delta.x
        guard abs(dx) > 1 else { return }

        let startX = origin.x
        let endX   = origin.x + dx
        let y      = origin.y

        var path = Path()
        path.move(to:    CGPoint(x: startX, y: y))
        path.addLine(to: CGPoint(x: endX,   y: y))

        // Amber/orange when dragging right (higher mag), blue when left (lower)
        let color: Color = dx > 0
            ? Color(red: 1.0, green: 0.72, blue: 0.25)   // amber
            : Color(red: 0.38, green: 0.63, blue: 1.0)   // blue

        context.stroke(path, with: .color(color.opacity(0.9)), lineWidth: 2.5)
    }

    private func drawDepBar(_ context: GraphicsContext) {
        let dy = delta.y
        guard abs(dy) > 1 else { return }

        let x      = origin.x
        let startY = origin.y
        let endY   = origin.y + dy

        var path = Path()
        path.move(to:    CGPoint(x: x, y: startY))
        path.addLine(to: CGPoint(x: x, y: endY))

        // Blue when dragging down (deeper), warm when dragging up (shallower)
        let color: Color = dy > 0
            ? Color(red: 0.38, green: 0.75, blue: 1.0)   // blue
            : Color(red: 1.0,  green: 0.63, blue: 0.31)  // warm orange

        context.stroke(path, with: .color(color.opacity(0.9)), lineWidth: 2.5)
    }
}

private struct FingerCircleView: View {

    let position: CGPoint

    var body: some View {
        Circle()
            .stroke(Color.white.opacity(0.6), lineWidth: 1.5)
            .frame(width: 22, height: 22)
            .position(position)
    }
}

private struct ParameterCardView: View {

    let parameters: QuakeParameters
    let fingerPoint: CGPoint

    /// Card dimensions for clamping — approximate, updated after first layout.
    private let cardWidth:  CGFloat = 148
    private let cardHeight: CGFloat = 96

    var body: some View {
        GeometryReader { geo in
            card
                .frame(width: cardWidth)
                .position(clampedPosition(in: geo.size))
        }
    }

    private var card: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("PARAMETERS")
                .font(.system(size: 9, weight: .medium, design: .monospaced))
                .foregroundStyle(.white.opacity(0.35))
                .kerning(1.2)

            HStack(alignment: .firstTextBaseline, spacing: 4) {
                Text("MAG")
                    .cardLabel()
                Text(parameters.magnitude, format: .number.precision(.fractionLength(1)))
                    .cardValue(color: Color(red: 1.0, green: 0.72, blue: 0.25))
                Text("M")
                    .cardUnit()
            }

            HStack(alignment: .firstTextBaseline, spacing: 4) {
                Text("DEP")
                    .cardLabel()
                Text(Int(parameters.depthKm).formatted())
                    .cardValue(color: Color(red: 0.38, green: 0.75, blue: 1.0))
                Text("km")
                    .cardUnit()
            }

            Divider()
                .overlay(Color.white.opacity(0.12))
                .padding(.vertical, 2)

            let intensity = parameters.epicentralIntensity
            HStack(spacing: 4) {
                Text("Intensity:")
                    .font(.system(size: 10, design: .monospaced))
                    .foregroundStyle(.white.opacity(0.35))
                Text(intensity.roman)
                    .font(.system(size: 11, weight: .medium, design: .monospaced))
                    .foregroundStyle(Color(intensity.colorName))
            }
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 10)
        .background {
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .fill(Color(white: 0.06).opacity(0.92))
                .overlay(
                    RoundedRectangle(cornerRadius: 10, style: .continuous)
                        .stroke(Color.white.opacity(0.14), lineWidth: 0.5)
                )
        }
    }

    private func clampedPosition(in size: CGSize) -> CGPoint {
        // Place card above-right of the finger
        var x = fingerPoint.x + 22
        var y = fingerPoint.y - cardHeight * 0.5 - 10

        // Clamp to screen bounds with margin
        let margin: CGFloat = 8
        x = max(cardWidth / 2 + margin,
                min(x, size.width  - cardWidth  / 2 - margin))
        y = max(cardHeight / 2 + margin,
                min(y, size.height - cardHeight / 2 - margin))

        return CGPoint(x: x, y: y)
    }
}

private extension Text {
    func hudAxisLabel() -> some View {
        self
            .font(.system(size: 9, weight: .regular, design: .monospaced))
            .foregroundStyle(Color.white.opacity(0.28))
            .kerning(0.8)
    }

    func cardLabel() -> some View {
        self
            .font(.system(size: 10, design: .monospaced))
            .foregroundStyle(Color.white.opacity(0.4))
            .frame(minWidth: 28, alignment: .leading)
    }

    func cardValue(color: Color) -> some View {
        self
            .font(.system(size: 20, weight: .medium, design: .monospaced))
            .foregroundStyle(color)
            .monospacedDigit()
    }

    func cardUnit() -> some View {
        self
            .font(.system(size: 10, design: .monospaced))
            .foregroundStyle(Color.white.opacity(0.4))
    }
}
