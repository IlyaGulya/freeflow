// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "Wrenflow",
    platforms: [.macOS(.v14)],
    dependencies: [
        .package(url: "https://github.com/FluidInference/FluidAudio.git", from: "0.7.9"),
    ],
    targets: [
        // C headers for the Rust UniFFI library
        .systemLibrary(
            name: "wrenflow_ffiFFI",
            path: "FFIModule"
        ),
        .executableTarget(
            name: "Wrenflow",
            dependencies: [
                .product(name: "FluidAudio", package: "FluidAudio"),
                "wrenflow_ffiFFI",
            ],
            path: "Sources",
            linkerSettings: [
                .unsafeFlags([
                    "-Lcore/target/debug",
                    "-lwrenflow_ffi",
                ]),
            ]
        ),
        .executableTarget(
            name: "WrenflowCLI",
            path: "CLI"
        ),
    ]
)
