import SwiftUI
import AppKit

/// Floating error toast — appears briefly when an error occurs.
/// Same visual style as the model download dialog.
struct ErrorToastView: View {
    let message: String
    var onDismiss: () -> Void

    @State private var appeared = false

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: "exclamationmark.circle.fill")
                .font(.system(size: 20))
                .foregroundColor(WrenflowStyle.red)

            VStack(alignment: .leading, spacing: 2) {
                Text("Something went wrong")
                    .font(WrenflowStyle.title(14))
                    .foregroundColor(WrenflowStyle.text)

                Text(message)
                    .font(WrenflowStyle.body(12))
                    .foregroundColor(WrenflowStyle.textSecondary)
                    .lineLimit(3)
            }

            Spacer(minLength: 0)

            Button(action: onDismiss) {
                Image(systemName: "xmark")
                    .font(.system(size: 10, weight: .medium))
                    .foregroundColor(WrenflowStyle.textTertiary)
                    .frame(width: 20, height: 20)
                    .background(Circle().fill(WrenflowStyle.text.opacity(0.06)))
            }
            .buttonStyle(.plain)
        }
        .padding(.horizontal, 18)
        .padding(.vertical, 14)
        .wrenflowPanel(width: 380)
        .opacity(appeared ? 1 : 0)
        .offset(y: appeared ? 0 : -8)
        .onAppear {
            withAnimation(.easeOut(duration: 0.2)) {
                appeared = true
            }
        }
    }
}
