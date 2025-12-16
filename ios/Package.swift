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
        .target(
            name: "SkjuIOS",
            path: "Sources/SkjuIOS"
        )
    ]
)
