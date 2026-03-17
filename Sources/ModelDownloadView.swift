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
                    downloadContent(progress: 0, status: "Preparing...")

                case .downloading(let progress):
                    downloadContent(progress: progress, status: downloadStatus(progress))

                case .compiling:
                    Text("Loading model")
                        .font(.system(size: 16, weight: .medium))
                        .foregroundColor(Color(white: 0.15))

                    HStack(spacing: 3) {
                        ForEach(0..<5, id: \.self) { i in
                            RoundedRectangle(cornerRadius: 1.5)
                                .fill(Color(white: 0.15).opacity(0.2))
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
                            .font(.system(size: 16, weight: .medium))
                            .foregroundColor(Color(red: 0.2, green: 0.7, blue: 0.4))
                        Text("Ready")
                            .font(.system(size: 16, weight: .medium))
                            .foregroundColor(Color(white: 0.15))
                    }

                case .error(let message):
                    Text("Download failed")
                        .font(.system(size: 16, weight: .medium))
                        .foregroundColor(Color(white: 0.15))

                    Text(message)
                        .font(.system(size: 14))
                        .foregroundColor(Color(white: 0.4))
                        .lineLimit(2)

                    Button("Retry") { localTranscriptionService.initialize() }
                        .font(.system(size: 14))
                }
            }

            Spacer(minLength: 0)
        }
        .padding(.horizontal, 24)
        .padding(.vertical, 20)
        .frame(width: 380)
        .fixedSize(horizontal: false, vertical: true)
        .background(Color(white: 0.96))
        .environment(\.colorScheme, .light)
    }

    // MARK: - Download content

    private func downloadContent(progress: Double, status: String) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Downloading model")
                .font(.system(size: 16, weight: .medium))
                .foregroundColor(Color(white: 0.15))

            // Progress bar
            GeometryReader { geo in
                ZStack(alignment: .leading) {
                    RoundedRectangle(cornerRadius: 2)
                        .fill(Color(white: 0.15).opacity(0.08))
                        .frame(height: 5)

                    RoundedRectangle(cornerRadius: 2)
                        .fill(Color(white: 0.15).opacity(0.45))
                        .frame(width: max(5, geo.size.width * CGFloat(progress)), height: 5)
                        .animation(.easeInOut(duration: 0.3), value: progress)
                }
            }
            .frame(height: 5)

            Text(status)
                .font(.system(size: 14, design: .monospaced))
                .foregroundColor(Color(white: 0.45))
        }
    }

    // MARK: - Helpers

    private func downloadStatus(_ progress: Double) -> String {
        let pct = Int(progress * 100)
        let mbDone = Int(640.0 * progress)
        return "\(mbDone) / 640 MB · \(pct)%"
    }

    private func barHeight(index: Int) -> CGFloat {
        let heights: [CGFloat] = [12, 18, 14, 18, 12]
        return wavePhase == 0 ? 6 : heights[index]
    }
}
