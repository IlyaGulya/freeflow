import Foundation
import os.log

#if canImport(wrenflow_ffiFFI)
import wrenflow_ffiFFI
#endif

private let ltLog = OSLog(subsystem: "me.gulya.wrenflow", category: "LocalTranscription")

struct DownloadProgressInfo: Equatable {
    var fraction: Double = 0
    var bytesDownloaded: UInt64 = 0
    var totalBytes: UInt64? = nil
    var currentFile: String = ""
}

enum LocalTranscriptionState: Equatable {
    case notLoaded
    case downloading(DownloadProgressInfo)
    case compiling
    case ready
    case error(String)

    var isReady: Bool {
        if case .ready = self { return true }
        return false
    }

    var isLoading: Bool {
        switch self {
        case .downloading, .compiling: return true
        default: return false
        }
    }
}

enum LocalTranscriptionError: LocalizedError {
    case modelNotReady
    case audioReadFailed(String)
    case transcriptionFailed(String)

    var errorDescription: String? {
        switch self {
        case .modelNotReady:
            return "Local transcription model is not ready"
        case .audioReadFailed(let msg):
            return "Failed to read audio: \(msg)"
        case .transcriptionFailed(let msg):
            return "Local transcription failed: \(msg)"
        }
    }
}

/// Local transcription using Rust parakeet-rs via FFI.
/// Downloads ONNX model from HuggingFace if not present, then loads.
final class LocalTranscriptionService: ObservableObject, @unchecked Sendable {
    @Published var state: LocalTranscriptionState = .notLoaded

    #if canImport(wrenflow_ffiFFI)
    private var engine: FfiLocalTranscriptionEngine?
    private var progressListener: SwiftModelProgressListener?
    #endif
    private var cancelled = false

    private var modelDir: String {
        let support = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
        return support.appendingPathComponent("Wrenflow/Models/parakeet-tdt").path
    }

    func cancel() {
        cancelled = true
        #if canImport(wrenflow_ffiFFI)
        engine?.cancelDownload()
        #endif
        state = .notLoaded
        os_log(.info, log: ltLog, "cancelled")
    }

    func initialize() {
        cancelled = false
        guard !state.isReady && !state.isLoading else { return }
        os_log(.info, log: ltLog, "initialize() — download + load via Rust")

        #if canImport(wrenflow_ffiFFI)
        let eng = FfiLocalTranscriptionEngine()
        self.engine = eng

        let listener = SwiftModelProgressListener { [weak self] newState in
            DispatchQueue.main.async {
                self?.handleStateUpdate(newState)
            }
        }
        self.progressListener = listener

        state = .downloading(DownloadProgressInfo())

        // Run on background thread (download + load are blocking)
        let modelDir = self.modelDir
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            // Download if needed
            if !eng.isModelDownloaded(modelDir: modelDir) {
                os_log(.info, log: ltLog, "Model not found, downloading...")
                if let error = eng.downloadModel(modelDir: modelDir, listener: listener) {
                    os_log(.error, log: ltLog, "Download failed: %{public}@", error)
                    DispatchQueue.main.async { self?.state = .error(error) }
                    return
                }
            }

            // Check cancel
            guard self?.cancelled != true else {
                os_log(.info, log: ltLog, "cancelled before load")
                return
            }

            // Load model
            DispatchQueue.main.async { self?.state = .compiling }
            os_log(.info, log: ltLog, "Loading model...")
            if let error = eng.loadModel(modelDir: modelDir) {
                os_log(.error, log: ltLog, "Load failed: %{public}@", error)
                DispatchQueue.main.async { self?.state = .error(error) }
                return
            }

            os_log(.info, log: ltLog, "Model ready")
            DispatchQueue.main.async { self?.state = .ready }
        }
        #else
        state = .error("Rust FFI not available")
        #endif
    }

    #if canImport(wrenflow_ffiFFI)
    private func handleStateUpdate(_ ffiState: ModelState) {
        switch ffiState {
        case .notDownloaded:
            state = .notLoaded
        case let .downloading(fraction, bytesDown, totalBytes, file):
            state = .downloading(DownloadProgressInfo(
                fraction: fraction,
                bytesDownloaded: bytesDown,
                totalBytes: totalBytes > 0 ? totalBytes : nil,
                currentFile: file
            ))
        case .loading:
            state = .compiling
        case .ready:
            state = .ready
        case let .error(msg):
            state = .error(msg)
        }
    }
    #endif

    func transcribe(fileURL: URL) async throws -> String {
        #if canImport(wrenflow_ffiFFI)
        guard let engine = engine, state.isReady else {
            throw LocalTranscriptionError.modelNotReady
        }

        os_log(.info, log: ltLog, "transcribe() for: %{public}@", fileURL.lastPathComponent)
        let start = CFAbsoluteTimeGetCurrent()

        let result = engine.transcribeFile(filePath: fileURL.path)
        if let error = result.error {
            throw LocalTranscriptionError.transcriptionFailed(error)
        }

        let elapsed = (CFAbsoluteTimeGetCurrent() - start) * 1000
        os_log(.info, log: ltLog, "done in %.1fms: '%{public}@'", elapsed, result.text)
        return result.text
        #else
        throw LocalTranscriptionError.transcriptionFailed("Rust FFI not available")
        #endif
    }
}

// MARK: - Progress listener bridge

#if canImport(wrenflow_ffiFFI)
final class SwiftModelProgressListener: FfiModelProgressListener {
    private let callback: (ModelState) -> Void

    init(callback: @escaping (ModelState) -> Void) {
        self.callback = callback
    }

    func onStateChanged(state: ModelState) {
        callback(state)
    }
}
#endif
