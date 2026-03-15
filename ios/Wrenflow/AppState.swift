import Foundation
import AVFoundation

/// Shared app state — bridges to Rust core via FFI
class AppState: ObservableObject {
    @Published var isRecording = false
    @Published var lastTranscript = ""
    @Published var statusText = "Ready"

    private let audioCapture = AudioCapture()

    func startRecording() {
        guard !isRecording else { return }
        isRecording = true
        statusText = "Recording..."
        audioCapture.start()
    }

    func stopRecording() {
        guard isRecording else { return }
        isRecording = false
        statusText = "Transcribing..."
        audioCapture.stop { [weak self] fileURL in
            // TODO: Call Rust core transcribe via FFI
            DispatchQueue.main.async {
                self?.statusText = "Ready"
                self?.lastTranscript = "Transcription will appear here"
            }
        }
    }
}
