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
    targets: [
        .binaryTarget(
            name: "SkjuCommon",
            path: "../common/target/SkjuCommon-build/SkjuCommon.xcframework"
        ),
        .executableTarget(
            name: "SkjuSimulator",
            dependencies: ["SkjuCommon"],
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
