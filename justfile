# Wrenflow build system

app_name := "Wrenflow Debug"
bundle_id := "me.gulya.wrenflow.debug"
build_dir := "build"
app_bundle := build_dir / app_name + ".app"
contents := app_bundle / "Contents"
macos_dir := contents / "MacOS"
resources := contents / "Resources"
codesign_identity := "-"
icon_source := "Resources/AppIcon-Source.png"
icon_icns := "Resources/AppIcon.icns"
rust_dir := "core"
ffi_lib := rust_dir / "target/debug/libwrenflow_ffi.a"
ffi_swift := "Sources/Generated/wrenflow_ffi.swift"
ffi_header := "Sources/Generated/wrenflow_ffiFFI.h"
ffi_modulemap := "Sources/Generated/wrenflow_ffiFFI.modulemap"

# Default: build everything
default: build

# Build the full app (Rust + Swift + bundle)
build: rust swift bundle

# Build only Rust core + FFI
rust:
    cd {{rust_dir}} && cargo build -p wrenflow-ffi

# Generate UniFFI Swift bindings
uniffi: rust
    @mkdir -p Sources/Generated
    cd {{rust_dir}} && cargo run -p uniffi-bindgen generate \
        --library target/debug/libwrenflow_ffi.dylib \
        --language swift \
        --out-dir ../Sources/Generated
    @echo "Generated UniFFI bindings in Sources/Generated/"

# Build Swift app
swift:
    swift build -c debug

# Create .app bundle
bundle: swift
    @mkdir -p "{{macos_dir}}" "{{resources}}"
    @cp "$(swift build -c debug --show-bin-path)/Wrenflow" "{{macos_dir}}/{{app_name}}"
    @cp Info.plist "{{contents}}/"
    @plutil -replace CFBundleName -string "{{app_name}}" "{{contents}}/Info.plist"
    @plutil -replace CFBundleDisplayName -string "{{app_name}}" "{{contents}}/Info.plist"
    @plutil -replace CFBundleExecutable -string "{{app_name}}" "{{contents}}/Info.plist"
    @plutil -replace CFBundleIdentifier -string "{{bundle_id}}" "{{contents}}/Info.plist"
    @cp {{icon_icns}} "{{resources}}/"
    swift build -c debug --product WrenflowCLI
    @cp "$(swift build -c debug --show-bin-path)/WrenflowCLI" "{{macos_dir}}/wrenflow"
    @codesign --force --sign "{{codesign_identity}}" --entitlements Wrenflow.entitlements "{{app_bundle}}"
    @echo "Built {{app_bundle}}"

# Build and run (kill existing instance first)
run: build
    -pkill -f "{{app_name}}" 2>/dev/null; sleep 0.5
    open "{{app_bundle}}"

# Run with setup wizard reset
run-setup: build
    defaults delete {{bundle_id}} 2>/dev/null; true
    -pkill -f "{{app_name}}" 2>/dev/null; sleep 0.5
    open "{{app_bundle}}"

# Build CLI tool only
cli:
    @mkdir -p "{{build_dir}}"
    swift build -c debug --product WrenflowCLI
    @cp "$(swift build -c debug --show-bin-path)/WrenflowCLI" "{{build_dir}}/wrenflow"
    @echo "Built {{build_dir}}/wrenflow"

# Install CLI to /usr/local/bin
install-cli: cli
    cp "{{build_dir}}/wrenflow" /usr/local/bin/wrenflow
    @echo "Installed to /usr/local/bin/wrenflow"

# Generate app icon from source PNG
icon:
    @mkdir -p {{build_dir}}/AppIcon.iconset
    @sips -z 16 16 {{icon_source}} --out {{build_dir}}/AppIcon.iconset/icon_16x16.png > /dev/null
    @sips -z 32 32 {{icon_source}} --out {{build_dir}}/AppIcon.iconset/icon_16x16@2x.png > /dev/null
    @sips -z 32 32 {{icon_source}} --out {{build_dir}}/AppIcon.iconset/icon_32x32.png > /dev/null
    @sips -z 64 64 {{icon_source}} --out {{build_dir}}/AppIcon.iconset/icon_32x32@2x.png > /dev/null
    @sips -z 128 128 {{icon_source}} --out {{build_dir}}/AppIcon.iconset/icon_128x128.png > /dev/null
    @sips -z 256 256 {{icon_source}} --out {{build_dir}}/AppIcon.iconset/icon_128x128@2x.png > /dev/null
    @sips -z 256 256 {{icon_source}} --out {{build_dir}}/AppIcon.iconset/icon_256x256.png > /dev/null
    @sips -z 512 512 {{icon_source}} --out {{build_dir}}/AppIcon.iconset/icon_256x256@2x.png > /dev/null
    @sips -z 512 512 {{icon_source}} --out {{build_dir}}/AppIcon.iconset/icon_512x512.png > /dev/null
    @sips -z 1024 1024 {{icon_source}} --out {{build_dir}}/AppIcon.iconset/icon_512x512@2x.png > /dev/null
    @iconutil -c icns -o {{icon_icns}} {{build_dir}}/AppIcon.iconset
    @rm -rf {{build_dir}}/AppIcon.iconset
    @echo "Generated {{icon_icns}}"

# Create DMG installer
dmg: build
    @rm -f {{build_dir}}/{{app_name}}.dmg
    @rm -rf {{build_dir}}/dmg-staging
    @mkdir -p {{build_dir}}/dmg-staging
    @cp -R "{{app_bundle}}" {{build_dir}}/dmg-staging/
    @create-dmg \
        --volname "{{app_name}}" \
        --volicon "{{icon_icns}}" \
        --window-pos 200 120 \
        --window-size 660 400 \
        --icon-size 128 \
        --icon "{{app_name}}.app" 180 170 \
        --hide-extension "{{app_name}}.app" \
        --icon "Applications" 480 170 \
        --no-internet-enable \
        {{build_dir}}/{{app_name}}.dmg \
        {{build_dir}}/dmg-staging
    @rm -rf {{build_dir}}/dmg-staging
    @echo "Created {{build_dir}}/{{app_name}}.dmg"

# Clean all build artifacts
clean:
    rm -rf {{build_dir}} .build
    cd {{rust_dir}} && cargo clean

# Check everything compiles
check:
    cd {{rust_dir}} && cargo check
    swift build -c debug
