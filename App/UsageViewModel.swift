import Foundation
import AppKit

@MainActor
final class UsageViewModel: ObservableObject {
    @Published var summary: UsageSummary?
    @Published var profile: ProfileResponse?
    @Published var stats: StatsSnapshot?
    @Published var lastFetchedAt: Date?
    @Published var isLoading = false
    @Published var lastError: String?
    @Published var rateLimitedUntil: Date?

    private let api = APIClient()
    private let store = SharedStore()
    private let statsService = StatsService()
    private var refreshTask: Task<Void, Never>?

    init() {
        loadCached()
        Task {
            await NotificationService.shared.requestAuthorizationIfNeeded()
            await refresh()
        }
        startAutoRefresh()
        observeWake()
    }

    private func observeWake() {
        NSWorkspace.shared.notificationCenter.addObserver(
            forName: NSWorkspace.didWakeNotification,
            object: nil, queue: .main
        ) { [weak self] _ in
            Task { @MainActor in
                self?.startAutoRefresh()
                await self?.refresh()
            }
        }
    }

    func loadCached() {
        if let cached = store.readUsage() {
            self.summary = UsageSummary(cached.usage)
            self.lastFetchedAt = cached.fetchedAt
        }
        if let cached = store.readProfile() {
            self.profile = cached.profile
        }
    }

    func refresh() async {
        if let until = rateLimitedUntil, until > Date() {
            return
        }

        isLoading = true
        defer { isLoading = false }

        guard let token = KeychainReader.readClaudeCodeToken() else {
            lastError = "Could not read Claude token from Keychain"
            return
        }

        do {
            let usage = try await api.fetchUsage(token: token)
            let cached = CachedUsage(usage: usage, fetchedAt: Date())
            try store.writeUsage(cached)

            let summary = UsageSummary(usage)
            self.summary = summary
            self.lastFetchedAt = cached.fetchedAt
            self.lastError = nil
            self.rateLimitedUntil = nil

            await NotificationService.shared.checkAndNotify(summary: summary)
        } catch APIError.rateLimited(let retry) {
            let wait = retry ?? 300
            self.rateLimitedUntil = Date().addingTimeInterval(wait)
            self.lastError = rateLimitErrorMessage(retryAfter: wait)
        } catch {
            lastError = error.localizedDescription
        }

        if let profileResp = try? await api.fetchProfile(token: token) {
            self.profile = profileResp
            try? store.writeProfile(CachedProfile(profile: profileResp, fetchedAt: Date()))
        }

        self.stats = await statsService.compute()
    }

    private func rateLimitErrorMessage(retryAfter: TimeInterval) -> String {
        let when = Date().addingTimeInterval(retryAfter)
        let f = DateFormatter()
        f.timeStyle = .short
        return "Rate limited by Anthropic · retrying at \(f.string(from: when))"
    }

    private func startAutoRefresh() {
        refreshTask?.cancel()
        refreshTask = Task { [weak self] in
            while !Task.isCancelled {
                try? await Task.sleep(nanoseconds: 300 * 1_000_000_000)
                await self?.refresh()
            }
        }
    }
}
