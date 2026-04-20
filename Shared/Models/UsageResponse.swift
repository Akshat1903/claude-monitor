import Foundation

public struct UsageResponse: Codable, Sendable {
    public let fiveHour: UsageBlock?
    public let sevenDay: UsageBlock?
    public let sevenDayOpus: UsageBlock?
    public let sevenDaySonnet: UsageBlock?
    public let sevenDayOmelette: UsageBlock?
    public let extraUsage: ExtraUsage?
}

public struct UsageBlock: Codable, Sendable {
    public let utilization: Double
    public let resetsAt: Date?
}

public struct ExtraUsage: Codable, Sendable {
    public let isEnabled: Bool
    public let monthlyLimit: Double?
    public let usedCredits: Double?
    public let utilization: Double?
    public let currency: String?
}
