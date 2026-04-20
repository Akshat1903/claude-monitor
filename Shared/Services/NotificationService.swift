import Foundation
import UserNotifications

public final class NotificationService: @unchecked Sendable {
    public static let shared = NotificationService()

    private let thresholds: [Int] = [50, 80, 95]
    private let defaults = UserDefaults.standard

    private init() {}

    public func requestAuthorizationIfNeeded() async {
        let center = UNUserNotificationCenter.current()
        let settings = await center.notificationSettings()
        guard settings.authorizationStatus == .notDetermined else { return }
        _ = try? await center.requestAuthorization(options: [.alert, .sound])
    }

    public func checkAndNotify(summary: UsageSummary) async {
        await maybeNotify(
            scope: "five_hour", label: "5-hour window",
            pct: summary.fiveHourPct, resetsAt: summary.fiveHourResetsAt
        )
        await maybeNotify(
            scope: "seven_day", label: "Weekly",
            pct: summary.sevenDayPct, resetsAt: summary.sevenDayResetsAt
        )
        await maybeNotify(
            scope: "claude_design", label: "Claude Design weekly",
            pct: summary.claudeDesignPct, resetsAt: summary.claudeDesignResetsAt
        )
    }

    private func maybeNotify(scope: String, label: String, pct: Double?, resetsAt: Date?) async {
        guard let pct, let resetsAt else { return }

        let lastResetKey = "alerts.\(scope).lastReset"
        let firedKey = "alerts.\(scope).fired"

        let storedReset = defaults.object(forKey: lastResetKey) as? Date
        if storedReset == nil || storedReset! != resetsAt {
            defaults.set(resetsAt, forKey: lastResetKey)
            defaults.set([Int](), forKey: firedKey)
        }

        var fired = (defaults.array(forKey: firedKey) as? [Int]) ?? []
        let toFire = thresholds.filter { Int(pct) >= $0 && !fired.contains($0) }
        guard !toFire.isEmpty else { return }

        for t in toFire {
            await postNotification(threshold: t, label: label, pct: pct, resetsAt: resetsAt)
            fired.append(t)
        }
        defaults.set(fired, forKey: firedKey)
    }

    private func postNotification(threshold: Int, label: String, pct: Double, resetsAt: Date) async {
        let center = UNUserNotificationCenter.current()
        let settings = await center.notificationSettings()
        guard settings.authorizationStatus == .authorized
                || settings.authorizationStatus == .provisional else { return }

        let content = UNMutableNotificationContent()
        content.title = iconFor(threshold) + " Claude usage at \(threshold)%"
        let remaining = DurationFormat.short(max(0, resetsAt.timeIntervalSinceNow))
        content.body = "\(label) is \(Int(pct))% used. Resets in \(remaining)."
        content.sound = threshold >= 95 ? .defaultCritical : .default

        let req = UNNotificationRequest(
            identifier: "claude-\(label)-\(threshold)-\(Int(resetsAt.timeIntervalSince1970))",
            content: content, trigger: nil
        )
        _ = try? await center.add(req)
    }

    private func iconFor(_ threshold: Int) -> String {
        switch threshold {
        case ..<80: return "🟡"
        case 80..<95: return "🟠"
        default: return "🔴"
        }
    }
}
