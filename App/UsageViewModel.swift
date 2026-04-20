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
        observeWakeAndNetwork()
    }

    private func observeWakeAndNetwork() {
        let wake = NSWorkspace.shared.notificationCenter
        wake.addObserver(
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
    }

    func refresh() async {
        isLoading = true
        defer { isLoading = false }

        guard let token = KeychainReader.readClaudeCodeToken() else {
            lastError = "Could not read Claude token from Keychain"
            return
        }

        do {
            async let usageT = api.fetchUsage(token: token)
            async let profileT = try? api.fetchProfile(token: token)

            let usage = try await usageT
            let profile = await profileT

            let cached = CachedUsage(usage: usage, fetchedAt: Date())
            try store.writeUsage(cached)

            let summary = UsageSummary(usage)
            self.summary = summary
            self.profile = profile
            self.lastFetchedAt = cached.fetchedAt
            self.lastError = nil

            await NotificationService.shared.checkAndNotify(summary: summary)
        } catch {
            lastError = error.localizedDescription
        }

        self.stats = await statsService.compute()
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
