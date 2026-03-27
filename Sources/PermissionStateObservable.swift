import Foundation
import Combine
import AVFoundation
import AppKit

// MARK: - Permission types (mirrors Rust wrenflow-core::platform)

enum PermissionKind: CaseIterable, Hashable, Identifiable {
    case microphone
    case accessibility
    case screenRecording

    var id: Self { self }

    var label: String {
        switch self {
        case .microphone: return "Microphone"
        case .accessibility: return "Accessibility"
        case .screenRecording: return "Screen Recording"
        }
    }

    var icon: String {
        switch self {
        case .microphone: return "mic.fill"
        case .accessibility: return "hand.raised.fill"
        case .screenRecording: return "camera.viewfinder"
        }
    }

    var description: String {
        switch self {
        case .microphone: return "Record audio for transcription"
        case .accessibility: return "Paste transcribed text into apps"
        case .screenRecording: return "Context-aware post-processing"
        }
    }

    var settingsURL: URL? {
        switch self {
        case .microphone: return URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone")
        case .accessibility: return URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        case .screenRecording: return URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
        }
    }
}

enum PermissionState: Equatable {
    case unknown
    case notGranted
    case requesting
    case granted
    case denied
    case notApplicable

    var isSatisfied: Bool {
        self == .granted || self == .notApplicable
    }
}

// MARK: - PermissionStateObservable (single source of truth)

final class PermissionStateObservable: ObservableObject {
    @Published private(set) var states: [PermissionKind: PermissionState] = [:]

    /// Required permissions — must be granted to record.
    let required: [PermissionKind] = [.microphone, .accessibility]
    /// Optional — app works without but with reduced features.
    let optional: [PermissionKind] = [.screenRecording]

    private var pollTimer: Timer?

    init() {
        // Initialize all to unknown
        for kind in PermissionKind.allCases {
            states[kind] = .unknown
        }
        // First check
        refresh()
    }

    // MARK: - Public API

    func get(_ kind: PermissionKind) -> PermissionState {
        states[kind] ?? .unknown
    }

    var allRequiredSatisfied: Bool {
        required.allSatisfy { get($0).isSatisfied }
    }

    var missingRequired: [PermissionKind] {
        required.filter { !get($0).isSatisfied }
    }

    /// Start polling (call once from AppDelegate).
    func startPolling() {
        pollTimer?.invalidate()
        print("[Permissions] polling started")
        pollTimer = Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { [weak self] _ in
            DispatchQueue.main.async { self?.refresh() }
        }
    }

    func stopPolling() {
        pollTimer?.invalidate()
        pollTimer = nil
    }

    /// User clicked "Grant".
    func request(_ kind: PermissionKind) {
        performRequest(kind)
    }

    /// Refresh from OS.
    func refresh() {
        var changed = false
        for kind in PermissionKind.allCases {
            let os = queryOS(kind)
            let new = mapOSStatus(os)
            if states[kind] != new {
                states[kind] = new
                changed = true
            }
        }
        if changed {
            print("[Permissions] changed: mic=\(states[.microphone]!) ax=\(states[.accessibility]!) sr=\(states[.screenRecording]!)")
            objectWillChange.send()
        }
    }

    // MARK: - OS queries (macOS-specific)

    private enum OSStatus {
        case granted, notGranted, denied, notApplicable
    }

    private func queryOS(_ kind: PermissionKind) -> OSStatus {
        switch kind {
        case .microphone:
            switch AVCaptureDevice.authorizationStatus(for: .audio) {
            case .authorized: return .granted
            case .notDetermined: return .notGranted
            case .denied, .restricted: return .denied
            @unknown default: return .notGranted
            }
        case .accessibility:
            return AXIsProcessTrusted() ? .granted : .notGranted
        case .screenRecording:
            return CGPreflightScreenCaptureAccess() ? .granted : .notGranted
        }
    }

    private func mapOSStatus(_ os: OSStatus) -> PermissionState {
        switch os {
        case .granted: return .granted
        case .notGranted: return .notGranted
        case .denied: return .denied
        case .notApplicable: return .notApplicable
        }
    }

    private func performRequest(_ kind: PermissionKind) {
        // Don't do anything if already granted
        if get(kind).isSatisfied { return }

        switch kind {
        case .microphone:
            let status = AVCaptureDevice.authorizationStatus(for: .audio)
            if status == .notDetermined {
                // First time — show system dialog
                AVCaptureDevice.requestAccess(for: .audio) { [weak self] granted in
                    DispatchQueue.main.async {
                        if granted {
                            self?.states[.microphone] = .granted
                            self?.objectWillChange.send()
                            print("[Permissions] mic granted via dialog")
                        }
                        self?.refresh()
                    }
                }
            } else if status == .denied || status == .restricted {
                // Already denied — open System Settings
                temporarilyLowerFloatingPanels()
                if let url = kind.settingsURL {
                    NSWorkspace.shared.open(url)
                }
            }
        case .accessibility:
            temporarilyLowerFloatingPanels()
            // Register in Accessibility list (prompt: false = no system dialog)
            let opts = [kAXTrustedCheckOptionPrompt.takeUnretainedValue(): false] as CFDictionary
            AXIsProcessTrustedWithOptions(opts)
            // Open Settings to the Accessibility pane
            if let url = kind.settingsURL {
                NSWorkspace.shared.open(url)
            }
        case .screenRecording:
            temporarilyLowerFloatingPanels()
            CGRequestScreenCaptureAccess()
            if let url = kind.settingsURL {
                DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
                    NSWorkspace.shared.open(url)
                }
            }
        }
    }

    /// Lower all floating panels so system dialogs appear on top.
    /// Panels stay lowered — they'll restore when permission is granted
    /// and the panel closes, or when user interacts with it.
    private func temporarilyLowerFloatingPanels() {
        let panels = NSApp.windows.compactMap { $0 as? NSPanel }.filter { $0.isFloatingPanel }
        for panel in panels {
            panel.level = .normal
        }
    }

    deinit {
        pollTimer?.invalidate()
    }
}
