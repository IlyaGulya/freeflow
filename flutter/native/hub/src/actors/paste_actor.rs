//! Paste actor — copies text to clipboard and simulates Cmd+V / Ctrl+V.

use arboard::Clipboard;
use enigo::{Enigo, Keyboard, Key, Settings, Direction};

/// Paste text into the frontmost application.
/// 1. Set clipboard content via arboard
/// 2. Simulate Cmd+V (macOS) or Ctrl+V (Windows/Linux) via enigo
pub fn paste_text(text: &str) -> Result<(), String> {
    // Set clipboard
    let mut clipboard = Clipboard::new().map_err(|e| format!("clipboard error: {e}"))?;
    clipboard
        .set_text(text)
        .map_err(|e| format!("clipboard set error: {e}"))?;

    // Small delay to let clipboard settle
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Simulate paste keystroke
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| format!("enigo error: {e}"))?;

    #[cfg(target_os = "macos")]
    {
        enigo.key(Key::Meta, Direction::Press).map_err(|e| format!("key error: {e}"))?;
        enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| format!("key error: {e}"))?;
        enigo.key(Key::Meta, Direction::Release).map_err(|e| format!("key error: {e}"))?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        enigo.key(Key::Control, Direction::Press).map_err(|e| format!("key error: {e}"))?;
        enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| format!("key error: {e}"))?;
        enigo.key(Key::Control, Direction::Release).map_err(|e| format!("key error: {e}"))?;
    }

    Ok(())
}
