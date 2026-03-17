// swift-tools-version: 5.9
import PackageDescription
import Foundation

let packageDir = URL(fileURLWithPath: #filePath).deletingLastPathComponent().path

let package = Package(
    name: "Wrenflow",
    platforms: [.macOS(.v14)],
    dependencies: [],
    targets: [
        .systemLibrary(
            name: "wrenflow_ffiFFI",
            path: "FFIModule"
        ),
        .executableTarget(
            name: "Wrenflow",
            dependencies: [
                "wrenflow_ffiFFI",
            ],
            path: "Sources",
            linkerSettings: [
                .unsafeFlags(["-L\(packageDir)/core/target/debug", "-lwrenflow_ffi"],
                             .when(configuration: .debug)),
                .unsafeFlags(["-L\(packageDir)/core/target/release", "-lwrenflow_ffi"],
                             .when(configuration: .release)),
            ]
        ),
        .executableTarget(
            name: "WrenflowCLI",
            path: "CLI"
        ),
    ]
)
