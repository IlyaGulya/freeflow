import UIKit

/// Custom keyboard extension with dictation button.
///
/// IMPORTANT: Keyboard extensions do NOT have microphone access.
/// The dictation flow:
/// 1. User taps mic button in keyboard
/// 2. Extension signals main app via App Groups shared UserDefaults
/// 3. Main app starts recording (it HAS mic access)
/// 4. Main app transcribes via Rust core, writes result to shared container
/// 5. Extension polls for result and inserts via textDocumentProxy
class KeyboardViewController: UIInputViewController {

    private var micButton: UIButton!
    private var statusLabel: UILabel!
    private var pollTimer: Timer?

    private let sharedDefaults = UserDefaults(suiteName: "group.me.gulya.wrenflow")

    override func viewDidLoad() {
        super.viewDidLoad()
        setupUI()
    }

    private func setupUI() {
        let stack = UIStackView()
        stack.axis = .horizontal
        stack.spacing = 8
        stack.alignment = .center
        stack.translatesAutoresizingMaskIntoConstraints = false

        // Mic/dictation button
        micButton = UIButton(type: .system)
        micButton.setImage(UIImage(systemName: "mic.fill"), for: .normal)
        micButton.tintColor = .systemBlue
        micButton.titleLabel?.font = .systemFont(ofSize: 24)
        micButton.addTarget(self, action: #selector(micTapped), for: .touchUpInside)
        micButton.widthAnchor.constraint(equalToConstant: 44).isActive = true
        micButton.heightAnchor.constraint(equalToConstant: 44).isActive = true

        // Status label
        statusLabel = UILabel()
        statusLabel.text = "Tap mic to dictate"
        statusLabel.font = .systemFont(ofSize: 13)
        statusLabel.textColor = .secondaryLabel

        // Next keyboard button
        let nextKeyboardButton = UIButton(type: .system)
        nextKeyboardButton.setTitle("🌐", for: .normal)
        nextKeyboardButton.addTarget(self, action: #selector(handleInputModeList(from:with:)), for: .allTouchEvents)

        stack.addArrangedSubview(nextKeyboardButton)
        stack.addArrangedSubview(micButton)
        stack.addArrangedSubview(statusLabel)

        view.addSubview(stack)
        NSLayoutConstraint.activate([
            stack.centerXAnchor.constraint(equalTo: view.centerXAnchor),
            stack.centerYAnchor.constraint(equalTo: view.centerYAnchor),
            view.heightAnchor.constraint(equalToConstant: 60),
        ])
    }

    @objc private func micTapped() {
        // Signal main app to start recording
        sharedDefaults?.set(true, forKey: "dictation_requested")
        sharedDefaults?.set(Date().timeIntervalSince1970, forKey: "dictation_request_time")
        sharedDefaults?.synchronize()

        statusLabel.text = "Recording..."
        micButton.tintColor = .systemRed

        // Poll for result
        pollTimer?.invalidate()
        pollTimer = Timer.scheduledTimer(withTimeInterval: 0.2, repeats: true) { [weak self] _ in
            self?.checkForResult()
        }

        // Timeout after 30 seconds
        DispatchQueue.main.asyncAfter(deadline: .now() + 30) { [weak self] in
            self?.pollTimer?.invalidate()
            self?.resetUI()
        }
    }

    private func checkForResult() {
        guard let result = sharedDefaults?.string(forKey: "dictation_result"),
              !result.isEmpty else { return }

        // Insert text at cursor
        textDocumentProxy.insertText(result)

        // Clear result
        sharedDefaults?.removeObject(forKey: "dictation_result")
        sharedDefaults?.removeObject(forKey: "dictation_requested")
        sharedDefaults?.synchronize()

        pollTimer?.invalidate()
        resetUI()
    }

    private func resetUI() {
        statusLabel.text = "Tap mic to dictate"
        micButton.tintColor = .systemBlue
    }
}
