import Foundation

public struct ProfileResponse: Codable, Sendable {
    public let account: Account
    public let organization: Organization

    public struct Account: Codable, Sendable {
        public let fullName: String?
        public let displayName: String?
        public let email: String?
        public let hasClaudeMax: Bool?
        public let hasClaudePro: Bool?
    }

    public struct Organization: Codable, Sendable {
        public let name: String?
        public let organizationType: String?
        public let rateLimitTier: String?
        public let subscriptionStatus: String?
        public let hasExtraUsageEnabled: Bool?
    }
}

public extension ProfileResponse {
    var planLabel: String {
        if account.hasClaudeMax == true {
            let tier = organization.rateLimitTier ?? ""
            if tier.contains("20x") { return "Claude Max 20x" }
            if tier.contains("5x")  { return "Claude Max 5x" }
            return "Claude Max"
        }
        if account.hasClaudePro == true { return "Claude Pro" }
        return "Claude"
    }

    var shortName: String {
        account.displayName
            ?? account.fullName
            ?? account.email
            ?? "—"
    }
}
