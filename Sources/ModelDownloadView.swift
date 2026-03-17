import SwiftUI

struct ModelDownloadView: View {
    @ObservedObject var localTranscriptionService: LocalTranscriptionService
    var onDismiss: () -> Void

    @State private var birdOffset: CGFloat = 0
    @State private var wavePhase: Double = 0

    var body: some View {
        HStack(spacing: 16) {
            // Bird icon (left)
            Image(nsImage: NSApp.applicationIconImage)
                .resizable()
                .frame(width: 44, height: 44)
                .opacity(0.5)
                .offset(y: birdOffset)
                .onAppear {
                    withAnimation(.easeInOut(duration: 2.0).repeatForever(autoreverses: true)) {
                        birdOffset = -3
                    }
                }

            // Content (right)
            VStack(alignment: .leading, spacing: 8) {
                switch localTranscriptionService.state {
                case .notLoaded:
                    downloadContent(info: DownloadProgressInfo(), status: "Preparing...")

                case .downloading(let info):
                    downloadContent(info: info, status: downloadStatus(info))

                case .compiling:
                    Text("Loading model")
                        .font(WrenflowStyle.title())
                        .foregroundColor(WrenflowStyle.text)

                    HStack(spacing: 3) {
                        ForEach(0..<5, id: \.self) { i in
                            RoundedRectangle(cornerRadius: 1.5)
                                .fill(WrenflowStyle.text.opacity(0.2))
                                .frame(width: 3, height: barHeight(index: i))
                                .animation(
                                    .easeInOut(duration: 0.5)
                                    .repeatForever(autoreverses: true)
                                    .delay(Double(i) * 0.1),
                                    value: wavePhase
                                )
                        }
                    }
                    .frame(height: 20)
                    .onAppear { wavePhase = 1 }

                case .ready:
                    HStack(spacing: 8) {
                        Image(systemName: "checkmark")
                            .font(WrenflowStyle.title())
                            .foregroundColor(WrenflowStyle.green)
                        Text("Ready")
                            .font(WrenflowStyle.title())
                            .foregroundColor(WrenflowStyle.text)
                    }

                case .error(let message):
                    Text("Download failed")
                        .font(WrenflowStyle.title())
                        .foregroundColor(WrenflowStyle.text)

                    Text(message)
                        .font(WrenflowStyle.body())
                        .foregroundColor(WrenflowStyle.textSecondary)
                        .lineLimit(2)

                    Button("Retry") { localTranscriptionService.initialize() }
                        .font(WrenflowStyle.body())
                }
            }

            Spacer(minLength: 0)

            // Cancel / close button (right edge)
            if isBusy {
                Button(action: onDismiss) {
                    Image(systemName: "xmark")
                        .font(.system(size: 11, weight: .medium))
                        .foregroundColor(WrenflowStyle.textTertiary)
                        .frame(width: 22, height: 22)
                        .background(Circle().fill(WrenflowStyle.text.opacity(0.06)))
                }
                .buttonStyle(.plain)
                .help("Cancel")
            }
        }
        .padding(.horizontal, 24)
        .padding(.vertical, 20)
        .wrenflowPanel()
    }

    // MARK: - Download content

    private func downloadContent(info: DownloadProgressInfo, status: String) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Downloading model")
                .font(WrenflowStyle.title())
                .foregroundColor(WrenflowStyle.text)

            WrenflowProgressBar(progress: min(info.fraction, 1.0))

            Text(status)
                .font(WrenflowStyle.mono())
                .foregroundColor(WrenflowStyle.textSecondary)
        }
    }

    // MARK: - Helpers

    private func downloadStatus(_ info: DownloadProgressInfo) -> String {
        let mbDown = Int(info.bytesDownloaded / 1_000_000)
        if let total = info.totalBytes {
            let mbTotal = Int(total / 1_000_000)
            let pct = min(Int(info.fraction * 100), 100)
            return "\(mbDown) / \(mbTotal) MB · \(pct)%"
        } else if mbDown > 0 {
            return "\(mbDown) MB"
        } else {
            return "Starting..."
        }
    }

    private func barHeight(index: Int) -> CGFloat {
        let heights: [CGFloat] = [12, 18, 14, 18, 12]
        return wavePhase == 0 ? 6 : heights[index]
    }

    private var isBusy: Bool {
        switch localTranscriptionService.state {
        case .notLoaded, .downloading, .compiling: return true
        default: return false
        }
    }
}
