import SwiftUI

@main
struct ClaudeWidgetApp: App {
    @StateObject private var viewModel = UsageViewModel()

    var body: some Scene {
        MenuBarExtra {
            MenuBarView(viewModel: viewModel)
        } label: {
            MenuBarLabel(viewModel: viewModel)
        }
        .menuBarExtraStyle(.window)
    }
}

struct MenuBarLabel: View {
    @ObservedObject var viewModel: UsageViewModel

    var body: some View {
        HStack(spacing: 4) {
            Image(systemName: "gauge.with.dots.needle.67percent")
            if let pct = viewModel.summary?.fiveHourPct {
                Text("\(Int(pct))%")
            }
        }
    }
}
