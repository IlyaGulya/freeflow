import SwiftUI

/// Reusable view showing permission rows. Used in wizard steps AND repair sheet.
struct PermissionGateView: View {
    @EnvironmentObject var permissions: PermissionStateObservable
    let kinds: [PermissionKind]

    var body: some View {
        VStack(spacing: 8) {
            ForEach(kinds) { kind in
                PermissionRow(kind: kind)
            }
        }
    }
}

struct PermissionRow: View {
    @EnvironmentObject var permissions: PermissionStateObservable
    let kind: PermissionKind

    private var state: PermissionState { permissions.get(kind) }

    var body: some View {
        HStack(spacing: 12) {
            // Status circle
            ZStack {
                if state.isSatisfied {
                    Circle().fill(Color.green).frame(width: 28, height: 28)
                    Image(systemName: "checkmark")
                        .font(.system(size: 12, weight: .bold))
                        .foregroundStyle(.white)
                } else if state == .requesting {
                    Circle().fill(Color.orange.opacity(0.15)).frame(width: 28, height: 28)
                    ProgressView().controlSize(.small)
                } else {
                    Circle().fill(Color.blue.opacity(0.1)).frame(width: 28, height: 28)
                    Image(systemName: kind.icon)
                        .font(.system(size: 12))
                        .foregroundStyle(.blue)
                }
            }

            VStack(alignment: .leading, spacing: 1) {
                Text(kind.label)
                    .font(.system(size: 13, weight: .medium))
                    .foregroundStyle(state.isSatisfied ? .secondary : .primary)
                Text(kind.description)
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary)
            }

            Spacer()

            if state.isSatisfied {
                Text("Granted")
                    .font(.system(size: 11, weight: .medium))
                    .foregroundStyle(.green)
            } else if state == .requesting {
                Text("Waiting...")
                    .font(.system(size: 11))
                    .foregroundStyle(.orange)
            } else if state == .denied {
                Button("Open Settings") { permissions.request(kind) }
                    .font(.system(size: 11))
            } else {
                Button("Grant Access") { permissions.request(kind) }
                    .font(.system(size: 11))
            }
        }
        .padding(10)
        .background(
            RoundedRectangle(cornerRadius: 8)
                .fill(state.isSatisfied ? Color.green.opacity(0.04) : Color(nsColor: .controlBackgroundColor))
        )
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(state.isSatisfied ? Color.green.opacity(0.15) : Color.primary.opacity(0.06), lineWidth: 1)
        )
    }
}
