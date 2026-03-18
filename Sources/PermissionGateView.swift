import SwiftUI

/// Reusable permission view — used in wizard AND as standalone repair panel.
struct PermissionGateView: View {
    @EnvironmentObject var permissions: PermissionStateObservable
    let kinds: [PermissionKind]
    var onDismiss: (() -> Void)? = nil

    @State private var showSuccess = false

    private var allDone: Bool {
        kinds.allSatisfy { permissions.get($0).isSatisfied }
    }

    var body: some View {
        VStack(spacing: 0) {
            if showSuccess {
                // Success animation
                VStack(spacing: 10) {
                    Image(systemName: "checkmark.circle.fill")
                        .font(.system(size: 36))
                        .foregroundColor(WrenflowStyle.green)
                        .scaleEffect(showSuccess ? 1.0 : 0.3)
                        .opacity(showSuccess ? 1.0 : 0)

                    Text("All set")
                        .font(WrenflowStyle.title(15))
                        .foregroundColor(WrenflowStyle.text)
                        .opacity(showSuccess ? 1.0 : 0)
                }
                .padding(.vertical, 24)
                .frame(maxWidth: .infinity)
                .transition(.opacity)
            } else {
                // Header
                HStack(spacing: 10) {
                    Image(nsImage: NSApp.applicationIconImage)
                        .resizable()
                        .frame(width: 32, height: 32)
                        .opacity(0.5)

                    VStack(alignment: .leading, spacing: 2) {
                        Text("Permissions Required")
                            .font(WrenflowStyle.title(15))
                            .foregroundColor(WrenflowStyle.text)
                        Text("Wrenflow needs access to work properly.")
                            .font(WrenflowStyle.caption(12))
                            .foregroundColor(WrenflowStyle.textSecondary)
                    }

                    Spacer()

                    if let dismiss = onDismiss {
                        Button(action: dismiss) {
                            Image(systemName: "xmark")
                                .font(.system(size: 10, weight: .medium))
                                .foregroundColor(WrenflowStyle.textTertiary)
                                .frame(width: 20, height: 20)
                                .background(Circle().fill(WrenflowStyle.text.opacity(0.06)))
                        }
                        .buttonStyle(.plain)
                    }
                }
                .padding(.horizontal, 14)
                .padding(.top, 14)
                .padding(.bottom, 10)

                Rectangle()
                    .fill(WrenflowStyle.border)
                    .frame(height: 1)

                VStack(spacing: 4) {
                    ForEach(kinds) { kind in
                        PermissionRow(kind: kind)
                    }
                }
                .padding(.horizontal, 10)
                .padding(.vertical, 10)
            }
        }
        .onChange(of: allDone) { done in
            if done {
                withAnimation(.spring(response: 0.4, dampingFraction: 0.7)) {
                    showSuccess = true
                }
            }
        }
    }
}

struct PermissionRow: View {
    @EnvironmentObject var permissions: PermissionStateObservable
    let kind: PermissionKind

    private var state: PermissionState { permissions.get(kind) }

    var body: some View {
        HStack(spacing: 10) {
            ZStack {
                if state.isSatisfied {
                    Circle().fill(WrenflowStyle.green.opacity(0.12)).frame(width: 28, height: 28)
                    Image(systemName: "checkmark")
                        .font(.system(size: 11, weight: .bold))
                        .foregroundColor(WrenflowStyle.green)
                } else {
                    Circle().fill(WrenflowStyle.text.opacity(0.05)).frame(width: 28, height: 28)
                    Image(systemName: kind.icon)
                        .font(.system(size: 12))
                        .foregroundColor(WrenflowStyle.textSecondary)
                }
            }

            VStack(alignment: .leading, spacing: 1) {
                Text(kind.label)
                    .font(WrenflowStyle.body(13))
                    .foregroundColor(state.isSatisfied ? WrenflowStyle.textSecondary : WrenflowStyle.text)
                Text(kind.description)
                    .font(WrenflowStyle.caption(11))
                    .foregroundColor(WrenflowStyle.textTertiary)
            }

            Spacer()

            if state.isSatisfied {
                Text("Granted")
                    .font(WrenflowStyle.caption(11))
                    .foregroundColor(WrenflowStyle.green)
            } else if state == .denied {
                Button("Open Settings") { permissions.request(kind) }
                    .font(WrenflowStyle.body(11))
            } else {
                Button("Grant") { permissions.request(kind) }
                    .font(WrenflowStyle.body(11))
            }
        }
        .padding(8)
        .background(
            RoundedRectangle(cornerRadius: 8)
                .fill(state.isSatisfied ? WrenflowStyle.green.opacity(0.03) : WrenflowStyle.surface)
        )
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(state.isSatisfied ? WrenflowStyle.green.opacity(0.12) : WrenflowStyle.border, lineWidth: 1)
        )
    }
}
