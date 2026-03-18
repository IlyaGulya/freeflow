import Foundation

struct GroqModel: Identifiable, Codable {
    let id: String
    let ownedBy: String

    enum CodingKeys: String, CodingKey {
        case id
        case ownedBy = "owned_by"
    }
}

enum GroqModelsService {
    static func fetchModels(apiKey: String, baseURL: String = "https://api.groq.com/openai/v1") async -> [GroqModel] {
        do {
            let ffiModels: [FfiGroqModel] = try await Task.detached {
                try ffiFetchGroqModels(apiKey: apiKey, baseUrl: baseURL)
            }.value
            return ffiModels.map { GroqModel(id: $0.id, ownedBy: $0.ownedBy) }
        } catch {
            return []
        }
    }
}
