import SwiftUI

struct ContentView: View {
    @EnvironmentObject var appState: AppState

    var body: some View {
        VStack(spacing: 24) {
            Text("Wrenflow")
                .font(.largeTitle.bold())

            Text(appState.statusText)
                .foregroundStyle(.secondary)

            Button(action: {
                if appState.isRecording {
                    appState.stopRecording()
                } else {
                    appState.startRecording()
                }
            }) {
                Image(systemName: appState.isRecording ? "stop.circle.fill" : "mic.circle.fill")
                    .font(.system(size: 72))
                    .foregroundStyle(appState.isRecording ? .red : .blue)
            }

            if !appState.lastTranscript.isEmpty {
                Text(appState.lastTranscript)
                    .font(.body)
                    .padding()
                    .background(Color(.systemGray6))
                    .cornerRadius(12)
            }
        }
        .padding()
    }
}
