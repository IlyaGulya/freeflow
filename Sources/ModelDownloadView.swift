import SwiftUI

struct ModelDownloadView: View {
    @ObservedObject var localTranscriptionService: LocalTranscriptionService
    var onDismiss: () -> Void

    @State private var birdOffset: CGFloat = 0
    @State private var wavePhase: Double = 0

    var body: some View {
        VStack(spacing: 16) {
            // Bird icon
            Image(nsImage: NSApp.applicationIconImage)
                .resizable()
                .frame(width: 48, height: 48)
                .opacity(0.55)
                .offset(y: birdOffset)
                .onAppear {
                    withAnimation(.easeInOut(duration: 2.0).repeatForever(autoreverses: true)) {
                        birdOffset = -3
                    }
                }

            // Content per state
            switch localTranscriptionService.state {
            case .notLoaded:
                downloadContent(progress: 0, status: "Preparing...")

            case .downloading(let progress):
                downloadContent(progress: progress, status: downloadStatus(progress))

            case .compiling:
                VStack(spacing: 8) {
                    Text("Loading model")
                        .font(.system(size: 14, weight: .medium))

                    HStack(spacing: 3) {
                        ForEach(0..<5, id: \.self) { i in
                            RoundedRectangle(cornerRadius: 1.5)
                                .fill(Color.primary.opacity(0.15))
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

                    Text("Optimizing for your device")
                        .font(.system(size: 13))
                        .foregroundStyle(.secondary)
                }

            case .ready:
                VStack(spacing: 6) {
                    Image(systemName: "checkmark")
                        .font(.system(size: 18, weight: .medium))
                        .foregroundStyle(.green)
                    Text("Ready")
                        .font(.system(size: 14, weight: .medium))
                }

            case .error(let message):
                VStack(spacing: 8) {
                    Text("Download failed")
                        .font(.system(size: 14, weight: .medium))

                    Text(message)
                        .font(.system(size: 13))
                        .foregroundStyle(.secondary)
                        .multilineTextAlignment(.center)
                        .lineLimit(3)

                    Button("Retry") { localTranscriptionService.initialize() }
                        .font(.system(size: 13))
                }
            }

            // Footer (only when not busy)
            if !isBusy {
                Button(footerLabel) { onDismiss() }
                    .font(.system(size: 13))
                    .foregroundStyle(.secondary)
                    .buttonStyle(.plain)
            }
        }
        .padding(.horizontal, 28)
        .padding(.vertical, 24)
        .frame(width: 320)
        .fixedSize(horizontal: false, vertical: true)
        .background(Color(nsColor: .windowBackgroundColor))
    }

    // MARK: - Download content

    private func downloadContent(progress: Double, status: String) -> some View {
        VStack(spacing: 10) {
            Text("Downloading model")
                .font(.system(size: 14, weight: .medium))

            // Progress bar
            GeometryReader { geo in
                ZStack(alignment: .leading) {
                    RoundedRectangle(cornerRadius: 2)
                        .fill(Color.primary.opacity(0.06))
                        .frame(height: 4)

                    RoundedRectangle(cornerRadius: 2)
                        .fill(Color.primary.opacity(0.4))
                        .frame(width: max(4, geo.size.width * CGFloat(progress)), height: 4)
                        .animation(.easeInOut(duration: 0.3), value: progress)
                }
            }
            .frame(height: 4)

            Text(status)
                .font(.system(size: 13, design: .monospaced))
                .foregroundStyle(.secondary)

            Text("Parakeet TDT · ~640 MB · one-time")
                .font(.system(size: 12))
                .foregroundStyle(.tertiary)
        }
    }

    // MARK: - Helpers

    private func downloadStatus(_ progress: Double) -> String {
        let pct = Int(progress * 100)
        let mbDone = Int(640.0 * progress)
        return "\(mbDone) / 640 MB  ·  \(pct)%"
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

    private var footerLabel: String {
        switch localTranscriptionService.state {
        case .ready: return "Done"
        case .error: return "Close"
        default: return "Hide"
        }
    }
}
