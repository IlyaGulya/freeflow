import Foundation
import os.log

private let ppLog = OSLog(subsystem: "me.gulya.wrenflow", category: "PostProcessing")

enum PostProcessingError: LocalizedError {
    case requestFailed(Int, String)
    case invalidResponse(String)
    case requestTimedOut(TimeInterval)

    var errorDescription: String? {
        switch self {
        case .requestFailed(let statusCode, let details):
            "Post-processing failed with status \(statusCode): \(details)"
        case .invalidResponse(let details):
            "Invalid post-processing response: \(details)"
        case .requestTimedOut(let seconds):
            "Post-processing timed out after \(Int(seconds))s"
        }
    }
}

struct SwiftPostProcessingResult {
    let transcript: String
    let prompt: String
    let reasoning: String
}

final class PostProcessingService {
    static let defaultSystemPrompt = """
You are a dictation post-processor. You clean up raw speech-to-text output for typing.

CRITICAL: Output MUST be in the SAME language as RAW_TRANSCRIPTION. If input is Russian, output Russian. If input is English, output English. NEVER translate to another language.

Rules:
- Add punctuation, capitalization, and formatting.
- Remove filler words (um, uh, like, you know) unless they carry meaning.
- Fix misspellings using context and custom vocabulary — only correct words already spoken, never insert new ones.
- Preserve tone, intent, and word choice exactly. Never censor, rephrase, or omit anything including profanity and slang.

Respond with JSON: {"text": "cleaned text", "reasoning": "brief explanation of changes made"}
If the input is empty or only noise, respond: {"text": "", "reasoning": "explanation"}
"""
    static let defaultSystemPromptDate = "2026-02-24"

    private let apiKey: String
    private let baseURL: String
    private let model: String

    static func validateAPIKey(_ key: String, baseURL: String = "https://api.groq.com/openai/v1") async -> Bool {
        let trimmed = key.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return false }

        var request = URLRequest(url: URL(string: "\(baseURL)/models")!)
        request.setValue("Bearer \(trimmed)", forHTTPHeaderField: "Authorization")

        do {
            let (_, response) = try await URLSession.shared.data(for: request)
            let status = (response as? HTTPURLResponse)?.statusCode ?? 0
            return status == 200
        } catch {
            return false
        }
    }

    init(apiKey: String, baseURL: String = "https://api.groq.com/openai/v1", model: String = "meta-llama/llama-4-scout-17b-16e-instruct") {
        self.apiKey = apiKey
        self.baseURL = baseURL
        self.model = model
    }

    func postProcess(
        transcript: String,
        context: AppContext,
        customVocabulary: String,
        customSystemPrompt: String = ""
    ) async throws -> SwiftPostProcessingResult {
        os_log(.info, log: ppLog, "postProcess() — using Rust FFI path")
        let apiKey = self.apiKey
        let model = self.model
        let baseURL = self.baseURL
        let contextSummary = context.contextSummary

        return try await Task.detached {
            do {
                let ffiResult = try ffiPostProcess(
                    apiKey: apiKey,
                    model: model,
                    transcript: transcript,
                    contextSummary: contextSummary,
                    customVocab: customVocabulary,
                    customSystemPrompt: customSystemPrompt,
                    baseUrl: baseURL
                )
                return SwiftPostProcessingResult(
                    transcript: ffiResult.transcript,
                    prompt: ffiResult.prompt,
                    reasoning: ffiResult.reasoning
                )
            } catch {
                throw PostProcessingError.invalidResponse(error.localizedDescription)
            }
        }.value
    }
}
