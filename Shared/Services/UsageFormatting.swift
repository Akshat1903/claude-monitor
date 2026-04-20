import Foundation

public enum PaceZone: Sendable {
    case chill, onTrack, hot

    public var label: String {
        switch self {
        case .chill: return "Chill"
        case .onTrack: return "On track"
        case .hot: return "Hot"
        }
    }
}

public struct UsageSummary: Sendable {
    public let fiveHourPct: Double?
    public let sevenDayPct: Double?
    public let claudeDesignPct: Double?
    public let fiveHourResetsAt: Date?
    public let sevenDayResetsAt: Date?
    public let claudeDesignResetsAt: Date?

    public init(_ r: UsageResponse) {
        self.fiveHourPct = r.fiveHour?.utilization
        self.sevenDayPct = r.sevenDay?.utilization
        self.claudeDesignPct = r.sevenDayOmelette?.utilization
        self.fiveHourResetsAt = r.fiveHour?.resetsAt
        self.sevenDayResetsAt = r.sevenDay?.resetsAt
        self.claudeDesignResetsAt = r.sevenDayOmelette?.resetsAt
    }

    public var fiveHourPace: PaceZone? {
        guard let pct = fiveHourPct, let reset = fiveHourResetsAt else { return nil }
        let windowStart = reset.addingTimeInterval(-5 * 3600)
        let elapsed = Date().timeIntervalSince(windowStart)
        let expectedPct = (elapsed / (5 * 3600)) * 100
        let delta = pct - expectedPct
        if delta < -10 { return .chill }
        if delta > 10 { return .hot }
        return .onTrack
    }

    public func timeUntilFiveHourReset(now: Date = Date()) -> TimeInterval? {
        guard let r = fiveHourResetsAt else { return nil }
        return max(0, r.timeIntervalSince(now))
    }

    public func timeUntilWeeklyReset(now: Date = Date()) -> TimeInterval? {
        guard let r = sevenDayResetsAt else { return nil }
        return max(0, r.timeIntervalSince(now))
    }
}

public enum DurationFormat {
    public static func short(_ seconds: TimeInterval) -> String {
        let total = Int(seconds)
        let h = total / 3600
        let m = (total % 3600) / 60
        if h >= 24 {
            let d = h / 24
            let rh = h % 24
            return rh == 0 ? "\(d)d" : "\(d)d \(rh)h"
        }
        if h > 0 { return "\(h)h \(m)m" }
        return "\(m)m"
    }
}
