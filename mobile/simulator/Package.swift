// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "SkjuSimulator",
    platforms: [
        .iOS(.v18)
    ],
    products: [
        .executable(name: "SkjuSimulator", targets: ["SkjuSimulator"]),
    ],
    dependencies: [
        .package(url: "https://github.com/adam-fowler/mqtt-nio.git", from: "2.13.0"),
    ],
    targets: [
        .binaryTarget(
            name: "SkjuCommon",
            path: "../common/target/SkjuCommon-build/SkjuCommon.xcframework"
        ),
        .executableTarget(
            name: "SkjuSimulator",
            dependencies: ["SkjuCommon", .product(name: "MQTTNIO", package: "mqtt-nio")],
            path: "src/SkjuSimulator",
            linkerSettings: [
                .linkedFramework("UIKit"),
                .linkedFramework("Foundation"),
                .linkedFramework("CoreGraphics"),
                .linkedFramework("CoreText"),
                .linkedFramework("CoreImage"),
                .linkedFramework("CoreAnimation")
            ]
        )
    ]
)
