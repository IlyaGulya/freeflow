// swift-tools-version: 5.9
import PackageDescription
import Foundation

let packageDir = URL(fileURLWithPath: #filePath).deletingLastPathComponent().path

let package = Package(
    name: "Wrenflow",
    platforms: [.macOS(.v14)],
    dependencies: [
        .package(url: "https://github.com/mxcl/Version.git", from: "2.0.0"),
    ],
    targets: [
        .systemLibrary(
            name: "wrenflow_ffiFFI",
            path: "FFIModule"
        ),
        .executableTarget(
            name: "Wrenflow",
            dependencies: [
                "wrenflow_ffiFFI",
                .product(name: "Version", package: "Version"),
            ],
            path: "Sources",
            linkerSettings: [
                .unsafeFlags(["\(packageDir)/core/target/debug/libwrenflow_ffi.a"],
                             .when(configuration: .debug)),
                .unsafeFlags(["\(packageDir)/core/target/release/libwrenflow_ffi.a"],
                             .when(configuration: .release)),
            ]
        ),
        .executableTarget(
            name: "WrenflowCLI",
            path: "CLI"
        ),
        .testTarget(
            name: "WrenflowTests",
            dependencies: [
                .product(name: "Version", package: "Version"),
            ],
            path: "Tests"
        ),
    ]
)
