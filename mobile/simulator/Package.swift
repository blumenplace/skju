// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "SkjuIOS",
    platforms: [
        .iOS(.v18)
    ],
    products: [
        .library(name: "SkjuIOS", targets: ["SkjuIOS"]),
    ],
    targets: [
        .binaryTarget(
            name: "CommontLib",
            path: "Binaries/CommontLib.xcframework"
        ),
        .target(
            name: "SkjuIOS",
            dependencies: ["CommontLib"],
            path: "Sources/SkjuIOS",
        )
    ]
)
