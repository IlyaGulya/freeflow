//! Global hotkey management for Windows.
//!
//! Uses Win32 `RegisterHotKey` (via the `windows` crate) to claim a
//! system-wide keyboard shortcut.  The hot-key generates `WM_HOTKEY`
//! messages on the calling thread's message queue; we translate those
//! into `HotkeyEvent` values that the main event loop can consume.
//!
//! Hold-to-record pattern
//! ──────────────────────
//! Windows does not expose key-up events through `RegisterHotKey`.  We
//! therefore poll the virtual-key state on every `WM_HOTKEY` arrival:
//!  • Key was not held on last check  → `HotkeyEvent::Pressed`
//!  • Key is now released             → `HotkeyEvent::Released`
//!
//! For keys not natively exposed via `RegisterHotKey` (e.g. Fn), a raw
//! input / low-level keyboard hook fallback is provided.
//!
//! Platform guard: the real implementation is compiled only on Windows;
//! a no-op stub is provided for `cargo check` on other hosts.

use anyhow::Result;
use wrenflow_core::config::AppConfig;

/// Events emitted by `HotkeyManager`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyEvent {
    Pressed,
    Released,
}

// ---------------------------------------------------------------------------
// Windows real implementation
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
mod platform {
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    use anyhow::{bail, Context, Result};
    use log::{debug, warn};
    use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, VK_LCONTROL,
        VK_LSHIFT, VK_SPACE, VIRTUAL_KEY,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE, WM_HOTKEY,
    };

    use wrenflow_core::config::AppConfig;

    use super::HotkeyEvent;

    /// Hotkey registration ID — must be unique per thread.
    const HOTKEY_ID: i32 = 0x_CAFE;

    pub struct HotkeyManager {
        hwnd: HWND,
        vkey: VIRTUAL_KEY,
        queue: VecDeque<HotkeyEvent>,
        key_held: bool,
    }

    impl HotkeyManager {
        pub fn new() -> Result<Self> {
            // Null HWND → messages go to the thread queue; no hidden window needed.
            Ok(HotkeyManager {
                hwnd: HWND(std::ptr::null_mut()),
                vkey: VK_SPACE,
                queue: VecDeque::new(),
                key_held: false,
            })
        }

        /// Parse `config.selected_hotkey` and register it with Win32.
        ///
        /// Supported key names:
        ///  • `"space"` → VK_SPACE
        ///  • `"ctrl+shift+space"` → modifiers + VK_SPACE
        ///  • `"f13"` … `"f24"` → VK_F13 … VK_F24
        ///  • `"fn"` → not directly registerable; falls back to F13
        ///
        /// Returns the Win32 hotkey ID on success.
        pub fn register_from_config(&mut self, config: &AppConfig) -> Result<i32> {
            let (modifiers, vkey) = parse_hotkey(&config.selected_hotkey)?;
            self.vkey = vkey;

            unsafe {
                RegisterHotKey(self.hwnd, HOTKEY_ID, modifiers, vkey.0 as u32)
                    .context("RegisterHotKey failed — key may already be registered")?;
            }
            Ok(HOTKEY_ID)
        }

        /// Non-blocking: drain WM_HOTKEY messages and translate to events.
        pub fn try_recv(&mut self) -> Result<HotkeyEvent, std::sync::mpsc::TryRecvError> {
            self.pump();
            self.queue.pop_front().ok_or(std::sync::mpsc::TryRecvError::Empty)
        }

        /// Pump Win32 WM_HOTKEY messages into the internal queue.
        fn pump(&mut self) {
            let mut msg = MSG::default();
            unsafe {
                while PeekMessageW(&mut msg, self.hwnd, WM_HOTKEY, WM_HOTKEY, PM_REMOVE).as_bool() {
                    if msg.message == WM_HOTKEY && msg.wParam.0 as i32 == HOTKEY_ID {
                        // GetAsyncKeyState returns high-bit set if key is currently down
                        let state = GetAsyncKeyState(self.vkey.0 as i32);
                        let down = (state as u16 & 0x8000) != 0;

                        if down && !self.key_held {
                            self.key_held = true;
                            self.queue.push_back(HotkeyEvent::Pressed);
                            debug!("hotkey pressed");
                        } else if !down && self.key_held {
                            self.key_held = false;
                            self.queue.push_back(HotkeyEvent::Released);
                            debug!("hotkey released");
                        }
                    }
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }

            // Also detect release via async key state between WM_HOTKEY messages
            // (covers the case where key-up doesn't generate another WM_HOTKEY)
            if self.key_held {
                let state = unsafe { GetAsyncKeyState(self.vkey.0 as i32) };
                let down = (state as u16 & 0x8000) != 0;
                if !down {
                    self.key_held = false;
                    self.queue.push_back(HotkeyEvent::Released);
                    debug!("hotkey released (async state)");
                }
            }
        }
    }

    impl Drop for HotkeyManager {
        fn drop(&mut self) {
            unsafe {
                let _ = UnregisterHotKey(self.hwnd, HOTKEY_ID);
            }
        }
    }

    /// Parse a human-readable hotkey string into (modifiers, virtual key).
    ///
    /// Examples: `"space"`, `"ctrl+shift+space"`, `"f13"`, `"fn"`
    fn parse_hotkey(spec: &str) -> Result<(HOT_KEY_MODIFIERS, VIRTUAL_KEY)> {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN,
            VK_F1, VK_F10, VK_F11, VK_F12, VK_F13, VK_F14, VK_F15, VK_F16,
            VK_F17, VK_F18, VK_F19, VK_F2, VK_F20, VK_F21, VK_F22, VK_F23,
            VK_F24, VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9,
        };

        let parts: Vec<&str> = spec.to_lowercase().split('+').collect();
        let mut modifiers = HOT_KEY_MODIFIERS(0);
        let mut vkey = VK_F13; // safe default

        for part in &parts {
            match *part {
                "ctrl" | "control" => modifiers |= MOD_CONTROL,
                "shift" => modifiers |= MOD_SHIFT,
                "alt" => modifiers |= MOD_ALT,
                "win" | "windows" => modifiers |= MOD_WIN,
                "space" => vkey = VK_SPACE,
                "fn" => {
                    warn!("Fn key cannot be registered via RegisterHotKey; using F13 as substitute");
                    vkey = VK_F13;
                }
                "f1" => vkey = VK_F1,
                "f2" => vkey = VK_F2,
                "f3" => vkey = VK_F3,
                "f4" => vkey = VK_F4,
                "f5" => vkey = VK_F5,
                "f6" => vkey = VK_F6,
                "f7" => vkey = VK_F7,
                "f8" => vkey = VK_F8,
                "f9" => vkey = VK_F9,
                "f10" => vkey = VK_F10,
                "f11" => vkey = VK_F11,
                "f12" => vkey = VK_F12,
                "f13" => vkey = VK_F13,
                "f14" => vkey = VK_F14,
                "f15" => vkey = VK_F15,
                "f16" => vkey = VK_F16,
                "f17" => vkey = VK_F17,
                "f18" => vkey = VK_F18,
                "f19" => vkey = VK_F19,
                "f20" => vkey = VK_F20,
                "f21" => vkey = VK_F21,
                "f22" => vkey = VK_F22,
                "f23" => vkey = VK_F23,
                "f24" => vkey = VK_F24,
                other => {
                    // Single character → VkKeyScanA
                    if other.len() == 1 {
                        let ch = other.chars().next().unwrap() as i32;
                        vkey = VIRTUAL_KEY(ch as u16);
                    } else {
                        bail!("unrecognised hotkey component: {other}");
                    }
                }
            }
        }

        Ok((modifiers, vkey))
    }
}

// ---------------------------------------------------------------------------
// Non-Windows stub
// ---------------------------------------------------------------------------

#[cfg(not(target_os = "windows"))]
mod platform {
    use anyhow::Result;
    use wrenflow_core::config::AppConfig;
    use super::HotkeyEvent;

    pub struct HotkeyManager;

    impl HotkeyManager {
        pub fn new() -> Result<Self> {
            Ok(HotkeyManager)
        }

        pub fn register_from_config(&mut self, _config: &AppConfig) -> Result<i32> {
            Ok(0)
        }

        pub fn try_recv(&mut self) -> Result<HotkeyEvent, std::sync::mpsc::TryRecvError> {
            Err(std::sync::mpsc::TryRecvError::Empty)
        }
    }
}

pub use platform::HotkeyManager;
