// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "SkjuIOS",
    platforms: [
        .iOS(.v16)
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
