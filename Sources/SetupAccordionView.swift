import SwiftUI
import AVFoundation
import Combine
import ServiceManagement

// MARK: - Setup Wizard (minimal floating card, no chrome)

struct SetupAccordionView: View {
    var onComplete: () -> Void
    @EnvironmentObject var appState: AppState
    @EnvironmentObject var permissionState: PermissionStateObservable

    // MARK: - Steps

    private enum SetupStep: Int, CaseIterable, Identifiable {
        case micPermission = 0
        case accessibility
        case screenRecording
        case hotkey
        case vocabulary
        case launchAtLogin
        case testTranscription

        var id: Int { rawValue }

        var title: String {
            switch self {
            case .micPermission: return "Microphone"
            case .accessibility: return "Accessibility"
            case .screenRecording: return "Screen Recording"
            case .hotkey: return "Push-to-Talk Key"
            case .vocabulary: return "Custom Vocabulary"
            case .launchAtLogin: return "Launch at Login"
            case .testTranscription: return "Test Transcription"
            }
        }

        var subtitle: String {
            switch self {
            case .micPermission: return "Wrenflow needs microphone access\nto record your voice."
            case .accessibility: return "Required to paste transcribed text\ninto the active app."
            case .screenRecording: return "Captures screen context for\nsmarter post-processing."
            case .hotkey: return "Choose which key you hold to dictate."
            case .vocabulary: return "Add names, acronyms, or jargon\nso transcription gets them right."
            case .launchAtLogin: return "Start Wrenflow automatically\nwhen you log in."
            case .testTranscription: return "Try a quick recording to make sure\neverything works."
            }
        }

        var icon: String {
            switch self {
            case .micPermission: return "mic.fill"
            case .accessibility: return "hand.raised.fill"
            case .screenRecording: return "camera.viewfinder"
            case .hotkey: return "keyboard.fill"
            case .vocabulary: return "text.book.closed.fill"
            case .launchAtLogin: return "sunrise.fill"
            case .testTranscription: return "waveform"
            }
        }

        var skippable: Bool {
            switch self {
            case .screenRecording, .testTranscription: return true
            default: return false
            }
        }
    }

    // MARK: - State

    @State private var activeStep: Int = 0
    @State private var skippedSteps: Set<SetupStep> = []
    @State private var customVocabularyInput = ""
    @State private var direction: Edge = .trailing

    private var micPermissionGranted: Bool { permissionState.get(.microphone).isSatisfied }
    private var accessibilityGranted: Bool { permissionState.get(.accessibility).isSatisfied }
    private var screenRecordingGranted: Bool { permissionState.get(.screenRecording).isSatisfied }

    private enum TestPhase: Equatable { case idle, recording, transcribing, done }
    @State private var testPhase: TestPhase = .idle
    @State private var testAudioLevel: Float = 0.0
    @State private var testTranscript = ""
    @State private var testError: String?
    @State private var testAudioLevelCancellable: AnyCancellable?
    @State private var permissionCheckTimer: Timer?

    private var steps: [SetupStep] {
        SetupStep.allCases.filter { step in
            if step == .screenRecording {
                return !appState.apiKey.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
            }
            return true
        }
    }

    private var allDone: Bool { activeStep >= steps.count }

    private var currentStep: SetupStep? {
        allDone ? nil : steps[activeStep]
    }

    private var isLastStep: Bool {
        !allDone && activeStep == steps.count - 1
    }

    // MARK: - Body

    var body: some View {
        VStack(spacing: 0) {
            // Content
            Group {
                if let step = currentStep {
                    stepView(for: step)
                        .id(step.id)
                        .transition(cardTransition)
                }
            }
            .animation(.easeInOut(duration: 0.25), value: activeStep)

            // Minimal footer: back arrow, dots, next arrow
            cardFooter
        }
        .frame(width: 340)
        .background(WrenflowStyle.surface)
        .clipShape(RoundedRectangle(cornerRadius: 12))
        .overlay(
            RoundedRectangle(cornerRadius: 12)
                .stroke(Color(white: 0.0).opacity(0.08), lineWidth: 0.5)
        )
        .shadow(color: Color.black.opacity(0.08), radius: 24, y: 8)
        .environment(\.colorScheme, .light)
        .onAppear {
            customVocabularyInput = appState.customVocabulary
            permissionState.startPolling()
            skipSatisfiedSteps()
            appState.localTranscriptionService.initialize()
            // Direct timer — bypasses SwiftUI reactivity issues with borderless windows
            permissionCheckTimer = Timer.scheduledTimer(withTimeInterval: 0.5, repeats: true) { _ in
                DispatchQueue.main.async {
                    let micState = permissionState.get(.microphone)
                    let axState = permissionState.get(.accessibility)
                    let srState = permissionState.get(.screenRecording)
                    let micOS = AVCaptureDevice.authorizationStatus(for: .audio)
                    print("[Wizard timer] step=\(activeStep) mic=\(micState) ax=\(axState) sr=\(srState) micOS=\(micOS.rawValue) micGranted=\(micPermissionGranted)")
                    let before = activeStep
                    skipSatisfiedSteps()
                    if activeStep != before {
                        print("[Wizard timer] advanced \(before) -> \(activeStep)")
                        NSApp.activate(ignoringOtherApps: true)
                        for w in NSApp.windows where w.isVisible {
                            w.orderFrontRegardless()
                        }
                    }
                }
            }
        }
        .onDisappear {
            permissionCheckTimer?.invalidate()
            permissionCheckTimer = nil
        }
    }

    private var cardTransition: AnyTransition {
        .asymmetric(
            insertion: .move(edge: direction).combined(with: .opacity),
            removal: .move(edge: direction == .trailing ? .leading : .trailing).combined(with: .opacity)
        )
    }

    // MARK: - Step View

    @ViewBuilder
    private func stepView(for step: SetupStep) -> some View {
        VStack(spacing: 0) {
            // Icon
            ZStack {
                Circle()
                    .fill(WrenflowStyle.text.opacity(0.05))
                    .frame(width: 40, height: 40)
                Image(systemName: step.icon)
                    .font(.system(size: 17))
                    .foregroundColor(WrenflowStyle.text.opacity(0.7))
            }
            .padding(.top, 24)

            // Title
            Text(step.title)
                .font(WrenflowStyle.title(16))
                .foregroundColor(WrenflowStyle.text)
                .padding(.top, 10)

            // Subtitle
            Text(step.subtitle)
                .font(WrenflowStyle.caption(12))
                .foregroundColor(WrenflowStyle.textSecondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal, 28)
                .padding(.top, 4)

            // Step-specific content
            stepContent(for: step)
                .padding(.top, 14)
                .padding(.horizontal, 24)
                .padding(.bottom, 20)
        }
    }

    // MARK: - Step Content

    @ViewBuilder
    private func stepContent(for step: SetupStep) -> some View {
        switch step {
        case .micPermission:
            permissionContent(granted: micPermissionGranted) {
                permissionState.request(.microphone)
            }
        case .accessibility:
            VStack(spacing: 6) {
                permissionContent(
                    granted: accessibilityGranted,
                    buttonLabel: "Open Settings"
                ) {
                    permissionState.request(.accessibility)
                }
                if !accessibilityGranted {
                    Text("If you rebuilt the app, remove and re-add it\nin Accessibility settings.")
                        .font(WrenflowStyle.caption(10))
                        .foregroundColor(WrenflowStyle.textTertiary)
                        .multilineTextAlignment(.center)

                    #if DEBUG
                    Button {
                        let bundlePath = Bundle.main.bundleURL.path
                        let script = "sleep 0.5; open \"\(bundlePath)\""
                        Process.launchedProcess(launchPath: "/bin/sh", arguments: ["-c", script])
                        NSApp.terminate(nil)
                    } label: {
                        Text("Restart App")
                            .font(WrenflowStyle.caption(10))
                            .foregroundColor(WrenflowStyle.textSecondary)
                    }
                    .buttonStyle(.plain)
                    #endif
                }
            }
        case .screenRecording:
            permissionContent(granted: screenRecordingGranted) {
                permissionState.request(.screenRecording)
            }
        case .hotkey:
            hotkeyContent
        case .vocabulary:
            vocabularyContent
        case .launchAtLogin:
            launchAtLoginContent
        case .testTranscription:
            testContent
        }
    }

    // MARK: - Permission content

    @ViewBuilder
    private func permissionContent(granted: Bool, buttonLabel: String = "Grant Access", action: @escaping () -> Void) -> some View {
        if granted {
            HStack(spacing: 5) {
                Image(systemName: "checkmark.circle.fill")
                    .font(.system(size: 13))
                    .foregroundColor(WrenflowStyle.green)
                Text("Granted")
                    .font(WrenflowStyle.body(12))
                    .foregroundColor(WrenflowStyle.green)
            }
        } else {
            Button(action: action) {
                Text(buttonLabel)
                    .font(WrenflowStyle.body(12))
                    .foregroundColor(WrenflowStyle.text)
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 8)
                    .background(
                        RoundedRectangle(cornerRadius: 8)
                            .fill(WrenflowStyle.text.opacity(0.06))
                    )
                    .overlay(
                        RoundedRectangle(cornerRadius: 8)
                            .stroke(WrenflowStyle.border, lineWidth: 1)
                    )
            }
            .buttonStyle(.plain)
        }
    }

    // MARK: - Hotkey

    private var hotkeyContent: some View {
        VStack(spacing: 3) {
            ForEach(HotkeyOption.allCases) { option in
                Button { appState.selectedHotkey = option } label: {
                    HStack(spacing: 8) {
                        Image(systemName: appState.selectedHotkey == option ? "checkmark.circle.fill" : "circle")
                            .font(.system(size: 13))
                            .foregroundColor(appState.selectedHotkey == option ? WrenflowStyle.text : WrenflowStyle.textTertiary)
                        Text(option.displayName)
                            .font(WrenflowStyle.body(12))
                            .foregroundColor(WrenflowStyle.text)
                        Spacer()
                    }
                    .padding(.vertical, 7)
                    .padding(.horizontal, 10)
                    .background(
                        RoundedRectangle(cornerRadius: 7)
                            .fill(appState.selectedHotkey == option ? WrenflowStyle.text.opacity(0.05) : Color.clear)
                    )
                    .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
            }
            if appState.selectedHotkey == .fnKey {
                Text("If Fn opens Emoji picker, change it to \"Do Nothing\" in System Settings.")
                    .font(WrenflowStyle.caption(10))
                    .foregroundColor(WrenflowStyle.textTertiary)
                    .padding(.top, 2)
            }
        }
    }

    // MARK: - Vocabulary

    private var vocabularyContent: some View {
        VStack(alignment: .leading, spacing: 4) {
            TextEditor(text: $customVocabularyInput)
                .font(WrenflowStyle.mono(11))
                .frame(height: 48)
                .padding(4)
                .background(WrenflowStyle.bg)
                .cornerRadius(7)
                .overlay(
                    RoundedRectangle(cornerRadius: 7)
                        .stroke(WrenflowStyle.border, lineWidth: 1)
                )

            Text("Comma, newline, or semicolon separated.")
                .font(WrenflowStyle.caption(10))
                .foregroundColor(WrenflowStyle.textTertiary)
        }
    }

    // MARK: - Launch at Login

    private var launchAtLoginContent: some View {
        HStack {
            Text("Start at login")
                .font(WrenflowStyle.body(12))
                .foregroundColor(WrenflowStyle.text)
            Spacer()
            GreenToggle(isOn: $appState.launchAtLogin)
        }
        .padding(.vertical, 8)
        .padding(.horizontal, 12)
        .background(
            RoundedRectangle(cornerRadius: 7)
                .fill(WrenflowStyle.text.opacity(0.03))
        )
        .overlay(
            RoundedRectangle(cornerRadius: 7)
                .stroke(WrenflowStyle.border, lineWidth: 1)
        )
    }

    // MARK: - Test

    private var testContent: some View {
        VStack(spacing: 6) {
            switch testPhase {
            case .idle:
                VStack(spacing: 2) {
                    Text("Hold **\(appState.selectedHotkey.displayName)** and speak.")
                        .font(WrenflowStyle.body(12))
                        .foregroundColor(WrenflowStyle.text)
                    Text("Release to transcribe.")
                        .font(WrenflowStyle.caption(11))
                        .foregroundColor(WrenflowStyle.textTertiary)
                }
            case .recording:
                HStack(spacing: 6) {
                    Circle().fill(WrenflowStyle.red).frame(width: 6, height: 6)
                    Text("Listening...")
                        .font(WrenflowStyle.body(12))
                        .foregroundColor(WrenflowStyle.text)
                }
            case .transcribing:
                HStack(spacing: 6) {
                    ProgressView().controlSize(.small)
                    Text("Transcribing...")
                        .font(WrenflowStyle.body(12))
                        .foregroundColor(WrenflowStyle.textSecondary)
                }
            case .done:
                if let error = testError {
                    HStack(spacing: 4) {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundColor(WrenflowStyle.red)
                            .font(.system(size: 11))
                        Text(error)
                            .font(WrenflowStyle.caption(11))
                            .foregroundColor(WrenflowStyle.red)
                    }
                } else if testTranscript.isEmpty {
                    HStack(spacing: 4) {
                        Image(systemName: "exclamationmark.circle")
                            .foregroundColor(WrenflowStyle.textSecondary)
                            .font(.system(size: 11))
                        Text("No speech detected. Try again.")
                            .font(WrenflowStyle.caption(11))
                            .foregroundColor(WrenflowStyle.textSecondary)
                    }
                } else {
                    VStack(alignment: .leading, spacing: 4) {
                        HStack(spacing: 4) {
                            Image(systemName: "checkmark.circle.fill")
                                .foregroundColor(WrenflowStyle.green)
                                .font(.system(size: 11))
                            Text("Success")
                                .font(WrenflowStyle.caption(11))
                                .foregroundColor(WrenflowStyle.green)
                        }
                        Text(testTranscript)
                            .font(WrenflowStyle.body(11))
                            .foregroundColor(WrenflowStyle.text)
                            .padding(6)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .background(WrenflowStyle.bg)
                            .cornerRadius(6)
                    }
                }
            }
        }
        .onAppear { startTestHotkeyMonitoring() }
        .onDisappear { stopTestHotkeyMonitoring() }
    }

    // MARK: - Footer

    private var cardFooter: some View {
        HStack {
            // Back
            if activeStep > 0 {
                Button {
                    direction = .leading
                    withAnimation { goBack() }
                } label: {
                    Text("Back")
                        .font(WrenflowStyle.body(12))
                        .foregroundColor(WrenflowStyle.textTertiary)
                }
                .buttonStyle(.plain)
            }

            Spacer()

            // Dot indicators
            HStack(spacing: 5) {
                ForEach(Array(steps.enumerated()), id: \.offset) { index, _ in
                    Circle()
                        .fill(index == activeStep
                              ? WrenflowStyle.text.opacity(0.5)
                              : index < activeStep
                                ? WrenflowStyle.green.opacity(0.5)
                                : WrenflowStyle.text.opacity(0.1))
                        .frame(width: index == activeStep ? 6 : 5, height: index == activeStep ? 6 : 5)
                }
            }

            Spacer()

            // Right side: Skip or Continue/Finish
            if let step = currentStep {
                if isLastStep {
                    footerButton("Finish") {
                        stopTestHotkeyMonitoring()
                        direction = .trailing
                        onComplete()
                    }
                } else if step.skippable {
                    HStack(spacing: 12) {
                        Button {
                            skippedSteps.insert(step)
                            direction = .trailing
                            advanceFrom(activeStep)
                        } label: {
                            Text("Skip")
                                .font(WrenflowStyle.body(12))
                                .foregroundColor(WrenflowStyle.textTertiary)
                        }
                        .buttonStyle(.plain)

                        footerButton("Continue") {
                            direction = .trailing
                            advanceFrom(activeStep)
                        }
                    }
                } else if !isPermissionStep(step) {
                    footerButton("Continue") {
                        direction = .trailing
                        advanceFrom(activeStep)
                    }
                }
            }
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
    }

    private func footerButton(_ label: String, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            Text(label)
                .font(WrenflowStyle.body(12))
                .foregroundColor(WrenflowStyle.text)
                .padding(.horizontal, 14)
                .padding(.vertical, 5)
                .background(
                    RoundedRectangle(cornerRadius: 6)
                        .fill(WrenflowStyle.text.opacity(0.06))
                )
                .overlay(
                    RoundedRectangle(cornerRadius: 6)
                        .stroke(WrenflowStyle.border, lineWidth: 1)
                )
        }
        .buttonStyle(.plain)
    }

    private func isPermissionStep(_ step: SetupStep) -> Bool {
        switch step {
        case .micPermission, .accessibility, .screenRecording: return true
        default: return false
        }
    }

    // MARK: - Navigation

    private func goBack() {
        if activeStep > 0 { activeStep -= 1 }
    }

    private func advanceFrom(_ index: Int) {
        let step = steps[index]
        switch step {
        case .vocabulary:
            appState.customVocabulary = customVocabularyInput.trimmingCharacters(in: .whitespacesAndNewlines)
        case .testTranscription:
            stopTestHotkeyMonitoring()
        default: break
        }
        goNext(from: index)
    }

    private func goNext(from index: Int) {
        activeStep = index + 1
    }

    private func skipSatisfiedSteps() {
        while !allDone {
            let step = steps[activeStep]
            let satisfied: Bool
            switch step {
            case .micPermission:
                satisfied = micPermissionGranted
                print("[skipSatisfied] step=micPermission micGranted=\(satisfied) rawState=\(permissionState.get(.microphone))")
            case .accessibility:
                satisfied = accessibilityGranted
                print("[skipSatisfied] step=accessibility axGranted=\(satisfied) rawState=\(permissionState.get(.accessibility))")
            default:
                print("[skipSatisfied] step=\(step) — not a permission step, stopping")
                return
            }
            guard satisfied else {
                print("[skipSatisfied] not satisfied, stopping")
                return
            }
            print("[skipSatisfied] satisfied, advancing from \(activeStep)")
            withAnimation(.easeInOut(duration: 0.3)) {
                activeStep += 1
            }
        }
    }

    // MARK: - Test helpers

    private func startTestHotkeyMonitoring() {
        print("[SetupWizard] startTestHotkeyMonitoring called, starting hotkey: \(appState.selectedHotkey)")
        appState.hotkeyManager.start(option: appState.selectedHotkey)
        appState.hotkeyManager.onKeyDown = {
            DispatchQueue.main.async {
                print("[SetupWizard] onKeyDown fired, testPhase=\(testPhase)")
                guard testPhase == .idle || testPhase == .done else { return }
                if testPhase == .done { resetTest() }
                let deviceUID = appState.selectedMicrophoneID
                let deviceId = (deviceUID.isEmpty || deviceUID == "default") ? nil : deviceUID
                let listener = SwiftAudioCaptureListener(
                    onRecordingReady: { },
                    onAudioLevel: { level in
                        DispatchQueue.main.async { testAudioLevel = level }
                    },
                    onError: { message in
                        DispatchQueue.main.async {
                            testError = message
                            withAnimation { testPhase = .done }
                        }
                    }
                )
                if let error = appState.audioCapture.startRecording(deviceId: deviceId, listener: listener) {
                    print("[SetupWizard] Recording error: \(error)")
                    testError = error
                    withAnimation { testPhase = .done }
                } else {
                    print("[SetupWizard] Recording started")
                    withAnimation { testPhase = .recording }
                }
            }
        }
        appState.hotkeyManager.onKeyUp = {
            DispatchQueue.main.async {
                print("[SetupWizard] onKeyUp fired, testPhase=\(testPhase)")
                guard testPhase == .recording else { return }
                let result = appState.audioCapture.stopRecording()
                testAudioLevelCancellable?.cancel(); testAudioLevel = 0
                withAnimation { testPhase = .transcribing }
                guard let filePath = result?.filePath else {
                    print("[SetupWizard] No audio file from recorder")
                    testError = "No audio recorded."
                    withAnimation { testPhase = .done }; return
                }
                let url = URL(fileURLWithPath: filePath)
                print("[SetupWizard] Transcribing \(url.lastPathComponent), model state: \(appState.localTranscriptionService.state)")
                Task {
                    do {
                        let t = try await appState.localTranscriptionService.transcribe(fileURL: url)
                        print("[SetupWizard] Transcription result: \(t.prefix(50))")
                        await MainActor.run { testTranscript = t; withAnimation { testPhase = .done } }
                    } catch {
                        await MainActor.run { testError = error.localizedDescription; withAnimation { testPhase = .done } }
                    }
                }
            }
        }
    }

    private func stopTestHotkeyMonitoring() {
        appState.hotkeyManager.stop()
        appState.hotkeyManager.onKeyDown = nil
        appState.hotkeyManager.onKeyUp = nil
    }

    private func resetTest() { testPhase = .idle; testTranscript = ""; testError = nil; testAudioLevel = 0 }
}

// MARK: - Green toggle (macOS .switch ignores .tint)

private struct GreenToggle: View {
    @Binding var isOn: Bool

    var body: some View {
        Capsule()
            .fill(isOn ? WrenflowStyle.green : WrenflowStyle.text.opacity(0.1))
            .frame(width: 36, height: 20)
            .overlay(alignment: isOn ? .trailing : .leading) {
                Circle()
                    .fill(.white)
                    .frame(width: 16, height: 16)
                    .shadow(color: .black.opacity(0.12), radius: 1, y: 1)
                    .padding(.horizontal, 2)
            }
            .animation(.easeInOut(duration: 0.15), value: isOn)
            .onTapGesture { isOn.toggle() }
    }
}
