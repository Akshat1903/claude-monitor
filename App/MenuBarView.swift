import SwiftUI

struct MenuBarView: View {
    @ObservedObject var viewModel: UsageViewModel

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            header

            if let profile = viewModel.profile {
                profileRow(profile)
            }

            if let stats = viewModel.stats {
                statsRow(stats)
            }

            if let summary = viewModel.summary {
                GaugeRow(title: "5-hour window",
                         pct: summary.fiveHourPct,
                         resetsAt: summary.fiveHourResetsAt,
                         pace: summary.fiveHourPace)
                GaugeRow(title: "Weekly",
                         pct: summary.sevenDayPct,
                         resetsAt: summary.sevenDayResetsAt,
                         pace: nil)
                if summary.claudeDesignPct != nil {
                    GaugeRow(title: "Claude Design · weekly",
                             pct: summary.claudeDesignPct,
                             resetsAt: summary.claudeDesignResetsAt,
                             pace: nil)
                }
            } else if viewModel.lastError == nil {
                ProgressView().frame(maxWidth: .infinity)
            }

            if let err = viewModel.lastError {
                errorBanner(err)
            }

            Divider()
            footer
        }
        .padding(16)
        .frame(width: 320)
        .onAppear {
            if let t = viewModel.lastFetchedAt,
               Date().timeIntervalSince(t) > 60 {
                Task { await viewModel.refresh() }
            }
        }
    }

    private var header: some View {
        HStack {
            Text("Claude Usage").font(.headline)
            Spacer()
            Button {
                Task { await viewModel.refresh() }
            } label: {
                Image(systemName: "arrow.clockwise")
            }
            .buttonStyle(.borderless)
            .disabled(viewModel.isLoading)
        }
    }

    private func profileRow(_ p: ProfileResponse) -> some View {
        HStack(spacing: 6) {
            Text(p.shortName)
                .font(.caption)
                .foregroundColor(.secondary)
                .lineLimit(1)
            Text("·").foregroundColor(.secondary).font(.caption)
            Text(p.planLabel)
                .font(.caption2)
                .fontWeight(.medium)
                .padding(.horizontal, 6).padding(.vertical, 2)
                .background(Color.accentColor.opacity(0.15))
                .foregroundColor(.accentColor)
                .clipShape(Capsule())
            if let status = p.organization.subscriptionStatus,
               status.lowercased() != "active" {
                Text(status.capitalized)
                    .font(.caption2)
                    .padding(.horizontal, 6).padding(.vertical, 2)
                    .background(Color.orange.opacity(0.2))
                    .foregroundColor(.orange)
                    .clipShape(Capsule())
            }
            Spacer()
        }
    }

    private func statsRow(_ s: StatsSnapshot) -> some View {
        HStack(spacing: 0) {
            statTile(value: TokenFormat.short(s.todayTokens), label: "Today")
            Divider().frame(height: 28)
            statTile(value: TokenFormat.short(s.weekTokens), label: "Week")
            Divider().frame(height: 28)
            statTile(value: s.favoriteModel ?? "—", label: "Favorite")
        }
        .padding(.vertical, 6)
        .background(Color.primary.opacity(0.04))
        .clipShape(RoundedRectangle(cornerRadius: 8))
    }

    private func statTile(value: String, label: String) -> some View {
        VStack(spacing: 2) {
            Text(value).font(.system(.callout, design: .rounded)).fontWeight(.semibold)
                .lineLimit(1).minimumScaleFactor(0.7)
            Text(label).font(.caption2).foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity)
    }

    private func errorBanner(_ msg: String) -> some View {
        HStack(alignment: .top, spacing: 6) {
            Image(systemName: "exclamationmark.triangle.fill")
                .foregroundColor(.orange)
                .font(.caption)
            Text(msg)
                .font(.caption)
                .foregroundColor(.primary.opacity(0.8))
                .fixedSize(horizontal: false, vertical: true)
            Spacer(minLength: 0)
        }
        .padding(.vertical, 6)
        .padding(.horizontal, 8)
        .background(Color.orange.opacity(0.12))
        .clipShape(RoundedRectangle(cornerRadius: 6))
    }

    private var footer: some View {
        HStack {
            if let t = viewModel.lastFetchedAt {
                Text("Updated \(t.formatted(.relative(presentation: .named)))")
                    .font(.caption2)
                    .foregroundColor(.secondary)
            }
            Spacer()
            Button("Quit") { NSApp.terminate(nil) }
                .buttonStyle(.borderless)
                .font(.caption)
        }
    }
}

struct GaugeRow: View {
    let title: String
    let pct: Double?
    let resetsAt: Date?
    let pace: PaceZone?

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(title).font(.subheadline).fontWeight(.medium)
                if let pace {
                    Text(pace.label)
                        .font(.caption2)
                        .padding(.horizontal, 6).padding(.vertical, 2)
                        .background(paceColor(pace).opacity(0.2))
                        .foregroundColor(paceColor(pace))
                        .clipShape(Capsule())
                }
                Spacer()
                if let pct { Text("\(Int(pct))%").monospacedDigit() }
            }
            ProgressView(value: (pct ?? 0) / 100)
                .tint(color(for: pct ?? 0))
            if let reset = resetsAt {
                Text("Resets in \(DurationFormat.short(max(0, reset.timeIntervalSinceNow)))")
                    .font(.caption2)
                    .foregroundColor(.secondary)
            }
        }
    }

    private func color(for pct: Double) -> Color {
        if pct >= 90 { return .red }
        if pct >= 70 { return .orange }
        return .accentColor
    }

    private func paceColor(_ p: PaceZone) -> Color {
        switch p {
        case .chill: return .green
        case .onTrack: return .blue
        case .hot: return .red
        }
    }
}
