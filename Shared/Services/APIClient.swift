import Foundation

public enum APIError: Error, LocalizedError {
    case invalidResponse
    case tokenExpired
    case rateLimited(retryAfter: TimeInterval?)
    case http(Int)

    public var errorDescription: String? {
        switch self {
        case .invalidResponse: return "Invalid response from server"
        case .tokenExpired: return "Auth token expired — run `claude /login`"
        case .rateLimited(let r): return "Rate limited\(r.map { " (retry in \(Int($0))s)" } ?? "")"
        case .http(let code): return "HTTP \(code)"
        }
    }
}

public final class APIClient: Sendable {
    private let usageURL = URL(string: "https://api.anthropic.com/api/oauth/usage")!
    private let profileURL = URL(string: "https://api.anthropic.com/api/oauth/profile")!
    private let userAgent: String

    public init(userAgent: String = "claude-code/1.0.0") {
        self.userAgent = userAgent
    }

    public func fetchUsage(token: String) async throws -> UsageResponse {
        try await get(url: usageURL, token: token, as: UsageResponse.self)
    }

    public func fetchProfile(token: String) async throws -> ProfileResponse {
        try await get(url: profileURL, token: token, as: ProfileResponse.self)
    }

    private func get<T: Decodable>(url: URL, token: String, as: T.Type) async throws -> T {
        var req = URLRequest(url: url)
        req.httpMethod = "GET"
        req.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        req.setValue("oauth-2025-04-20", forHTTPHeaderField: "anthropic-beta")
        req.setValue(userAgent, forHTTPHeaderField: "User-Agent")

        let (data, response) = try await URLSession.shared.data(for: req)
        guard let http = response as? HTTPURLResponse else { throw APIError.invalidResponse }

        switch http.statusCode {
        case 200:
            return try Self.decoder.decode(T.self, from: data)
        case 401, 403:
            throw APIError.tokenExpired
        case 429:
            let retry = http.value(forHTTPHeaderField: "Retry-After").flatMap(TimeInterval.init)
            throw APIError.rateLimited(retryAfter: retry)
        default:
            throw APIError.http(http.statusCode)
        }
    }

    static let decoder: JSONDecoder = {
        let d = JSONDecoder()
        d.keyDecodingStrategy = .convertFromSnakeCase
        d.dateDecodingStrategy = .custom { decoder in
            let c = try decoder.singleValueContainer()
            let str = try c.decode(String.self)
            let f = ISO8601DateFormatter()
            f.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
            if let d = f.date(from: str) { return d }
            f.formatOptions = [.withInternetDateTime]
            if let d = f.date(from: str) { return d }
            throw DecodingError.dataCorruptedError(in: c, debugDescription: "Unparseable date: \(str)")
        }
        return d
    }()
}
