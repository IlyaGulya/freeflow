import AVFoundation
import Foundation

/// iOS audio capture using AVAudioEngine.
/// Records 16kHz mono float32 and saves to WAV.
class AudioCapture {
    private var audioEngine: AVAudioEngine?
    private var audioFile: AVAudioFile?
    private var tempFileURL: URL?

    func start() {
        let engine = AVAudioEngine()
        let inputNode = engine.inputNode

        // Configure audio session
        let session = AVAudioSession.sharedInstance()
        try? session.setCategory(.record, mode: .measurement)
        try? session.setActive(true)

        // Output format: 16kHz mono
        let format = AVAudioFormat(commonFormat: .pcmFormatFloat32, sampleRate: 16000, channels: 1, interleaved: false)!
        let tempURL = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString + ".wav")

        let file = try? AVAudioFile(forWriting: tempURL, settings: [
            AVFormatIDKey: kAudioFormatLinearPCM,
            AVSampleRateKey: 16000,
            AVNumberOfChannelsKey: 1,
            AVLinearPCMBitDepthKey: 16,
            AVLinearPCMIsFloatKey: false,
        ])

        // Install tap with conversion to 16kHz
        let inputFormat = inputNode.outputFormat(forBus: 0)
        let converter = AVAudioConverter(from: inputFormat, to: format)

        inputNode.installTap(onBus: 0, bufferSize: 4096, format: inputFormat) { [weak self] buffer, _ in
            guard let converter, let file else { return }
            let convertedBuffer = AVAudioPCMBuffer(pcmFormat: format, frameCapacity: 4096)!
            var error: NSError?
            converter.convert(to: convertedBuffer, error: &error) { _, outStatus in
                outStatus.pointee = .haveData
                return buffer
            }
            if convertedBuffer.frameLength > 0 {
                try? file.write(from: convertedBuffer)
            }
        }

        try? engine.start()
        self.audioEngine = engine
        self.audioFile = file
        self.tempFileURL = tempURL
    }

    func stop(completion: @escaping (URL?) -> Void) {
        audioEngine?.inputNode.removeTap(onBus: 0)
        audioEngine?.stop()
        audioEngine = nil
        audioFile = nil
        completion(tempFileURL)
    }

    func cleanup() {
        if let url = tempFileURL {
            try? FileManager.default.removeItem(at: url)
            tempFileURL = nil
        }
    }
}
