import Foundation

public enum SharedPaths {
    public static var supportDir: URL {
        let home = FileManager.default.homeDirectoryForCurrentUser
        return home.appendingPathComponent("Library/Application Support/ClaudeWidget")
    }
    public static var usageFile: URL { supportDir.appendingPathComponent("usage.json") }
}

public struct CachedUsage: Codable, Sendable {
    public let usage: UsageResponse
    public let fetchedAt: Date
}

public struct SharedStore: Sendable {
    public init() {}

    public func readUsage() -> CachedUsage? {
        guard let data = try? Data(contentsOf: SharedPaths.usageFile) else { return nil }
        return try? Self.decoder.decode(CachedUsage.self, from: data)
    }

    public func writeUsage(_ cached: CachedUsage) throws {
        try FileManager.default.createDirectory(
            at: SharedPaths.supportDir, withIntermediateDirectories: true
        )
        let data = try Self.encoder.encode(cached)
        try data.write(to: SharedPaths.usageFile, options: .atomic)
    }

    static let encoder: JSONEncoder = {
        let e = JSONEncoder()
        e.dateEncodingStrategy = .iso8601
        return e
    }()

    static let decoder: JSONDecoder = {
        let d = JSONDecoder()
        d.dateDecodingStrategy = .iso8601
        return d
    }()
}
