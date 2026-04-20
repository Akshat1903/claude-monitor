import Foundation

public struct StatsSnapshot: Codable, Sendable {
    public let todayTokens: Int
    public let weekTokens: Int
    public let favoriteModel: String?
}

public struct StatsService: Sendable {
    public init() {}

    public func compute() async -> StatsSnapshot {
        await Task.detached(priority: .utility) {
            Self.computeSync()
        }.value
    }

    private static func computeSync() -> StatsSnapshot {
        let home = FileManager.default.homeDirectoryForCurrentUser
        let root = home.appendingPathComponent(".claude/projects")
        let files = collectJSONLFiles(under: root)

        let now = Date()
        let startOfToday = Calendar.current.startOfDay(for: now)
        let weekAgo = now.addingTimeInterval(-7 * 24 * 3600)

        var todayTokens = 0
        var weekTokens = 0
        var modelTokens: [String: Int] = [:]

        for url in files {
            guard let contents = try? String(contentsOf: url, encoding: .utf8) else { continue }
            for line in contents.split(separator: "\n", omittingEmptySubsequences: true) {
                guard let data = line.data(using: .utf8),
                      let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                      obj["type"] as? String == "assistant",
                      let ts = obj["timestamp"] as? String,
                      let date = parseTimestamp(ts),
                      date >= weekAgo,
                      let message = obj["message"] as? [String: Any],
                      let usage = message["usage"] as? [String: Any] else { continue }

                let input = usage["input_tokens"] as? Int ?? 0
                let output = usage["output_tokens"] as? Int ?? 0
                let cacheCreate = usage["cache_creation_input_tokens"] as? Int ?? 0
                let total = input + output + cacheCreate

                weekTokens += total
                if date >= startOfToday { todayTokens += total }
                if let model = message["model"] as? String {
                    modelTokens[model, default: 0] += total
                }
            }
        }

        let favoriteRaw = modelTokens.max(by: { $0.value < $1.value })?.key
        return StatsSnapshot(
            todayTokens: todayTokens,
            weekTokens: weekTokens,
            favoriteModel: favoriteRaw.map(displayModelName)
        )
    }

    private static func collectJSONLFiles(under root: URL) -> [URL] {
        guard let enumerator = FileManager.default.enumerator(
            at: root, includingPropertiesForKeys: nil,
            options: [.skipsHiddenFiles]
        ) else { return [] }
        var result: [URL] = []
        for case let url as URL in enumerator where url.pathExtension == "jsonl" {
            result.append(url)
        }
        return result
    }

    private static let fractionalISO: ISO8601DateFormatter = {
        let f = ISO8601DateFormatter()
        f.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        return f
    }()
    private static let plainISO = ISO8601DateFormatter()

    private static func parseTimestamp(_ s: String) -> Date? {
        fractionalISO.date(from: s) ?? plainISO.date(from: s)
    }

    private static func displayModelName(_ raw: String) -> String {
        let parts = raw.split(separator: "-")
        guard parts.count >= 3, parts.first == "claude" else { return raw }
        let family = parts[1].capitalized
        let major = parts[2]
        let minor = parts.count >= 4 ? String(parts[3]) : ""
        let isMinorNumeric = !minor.isEmpty && minor.allSatisfy(\.isNumber) && minor.count <= 2
        let version = isMinorNumeric ? "\(major).\(minor)" : "\(major)"
        return "\(family) \(version)"
    }
}

public enum TokenFormat {
    public static func short(_ n: Int) -> String {
        if n >= 1_000_000 {
            let v = Double(n) / 1_000_000
            return String(format: v >= 10 ? "%.0fm" : "%.1fm", v)
        }
        if n >= 1_000 {
            let v = Double(n) / 1_000
            return String(format: v >= 10 ? "%.0fk" : "%.1fk", v)
        }
        return "\(n)"
    }
}
