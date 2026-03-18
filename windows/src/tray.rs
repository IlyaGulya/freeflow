//! System tray icon and context menu.
//!
//! Uses Win32 `Shell_NotifyIcon` (via the `windows` crate) to create a
//! notification-area icon with a right-click context menu.
//!
//! Menu items
//! ──────────
//!   • Start / Stop  (greyed out when opposite action is unavailable)
//!   • Settings…
//!   • ─────────────
//!   • Quit
//!
//! Commands are sent through a `tokio::sync::mpsc::UnboundedSender<TrayCommand>`
//! so that the main event loop can act on them without blocking the message pump.
//!
//! Platform guard: the real implementation is compiled only on Windows;
//! a no-op stub is provided for `cargo check` on other hosts.

#[allow(unused_imports)]
use anyhow::Result;
#[allow(unused_imports)]
use tokio::sync::mpsc::UnboundedSender;

/// Commands that the tray menu can issue to the main event loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TrayCommand {
    OpenSettings,
    Quit,
}

// ---------------------------------------------------------------------------
// Windows real implementation
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
mod platform {
    use std::mem;

    use anyhow::{Context, Result};
    use log::debug;
    use tokio::sync::mpsc::UnboundedSender;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::Shell::{
        Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu,
        DispatchMessageW, GetMessageW, LoadIconW, PeekMessageW, PostQuitMessage,
        RegisterClassW, SetForegroundWindow, TrackPopupMenu, TranslateMessage,
        CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, IDI_APPLICATION, MF_SEPARATOR, MF_STRING,
        MSG, PM_REMOVE, TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_RETURNCMD,
        WM_APP, WM_COMMAND, WM_DESTROY, WM_RBUTTONUP, WNDCLASSW, WS_OVERLAPPEDWINDOW,
        HMENU,
    };

    use super::{TrayCommand, UnboundedSender};

    /// Custom message posted by Shell_NotifyIcon for tray events.
    const WM_TRAY: u32 = WM_APP + 1;

    /// Context menu item IDs.
    const IDM_SETTINGS: u32 = 1001;
    const IDM_QUIT: u32 = 1002;

    pub struct TrayIcon {
        hwnd: HWND,
        nid: NOTIFYICONDATAW,
        tx: UnboundedSender<TrayCommand>,
    }

    impl TrayIcon {
        pub fn new(tx: UnboundedSender<TrayCommand>) -> Result<Self> {
            let hwnd = create_message_window()?;
            let nid = build_nid(hwnd, WM_TRAY);
            Ok(TrayIcon { hwnd, nid, tx })
        }

        pub fn show(&mut self) -> Result<()> {
            unsafe {
                Shell_NotifyIconW(NIM_ADD, &self.nid)
                    .context("Shell_NotifyIconW NIM_ADD failed")?;
            }
            debug!("tray icon shown");
            Ok(())
        }

        pub fn hide(&mut self) -> Result<()> {
            unsafe {
                Shell_NotifyIconW(NIM_DELETE, &self.nid)
                    .context("Shell_NotifyIconW NIM_DELETE failed")?;
            }
            debug!("tray icon hidden");
            Ok(())
        }

        /// Pump the Win32 message queue once (non-blocking).
        /// Must be called regularly from the main event loop.
        pub fn pump_messages(&self) {
            let mut msg = MSG::default();
            unsafe {
                while PeekMessageW(&mut msg, self.hwnd, 0, 0, PM_REMOVE).as_bool() {
                    match msg.message {
                        WM_TRAY if msg.lParam.0 as u32 == WM_RBUTTONUP => {
                            self.show_context_menu();
                        }
                        WM_COMMAND => {
                            let item = (msg.wParam.0 & 0xFFFF) as u32;
                            match item {
                                IDM_SETTINGS => {
                                    let _ = self.tx.send(TrayCommand::OpenSettings);
                                }
                                IDM_QUIT => {
                                    let _ = self.tx.send(TrayCommand::Quit);
                                }
                                _ => {}
                            }
                        }
                        _ => {
                            TranslateMessage(&msg);
                            DispatchMessageW(&msg);
                        }
                    }
                }
            }
        }

        fn show_context_menu(&self) {
            unsafe {
                let hmenu = CreatePopupMenu().unwrap_or_default();
                if hmenu.is_invalid() {
                    return;
                }

                let settings_label: Vec<u16> = "Settings…\0".encode_utf16().collect();
                let quit_label: Vec<u16> = "Quit Wrenflow\0".encode_utf16().collect();

                AppendMenuW(hmenu, MF_STRING, IDM_SETTINGS as usize, PCWSTR(settings_label.as_ptr())).ok();
                AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null()).ok();
                AppendMenuW(hmenu, MF_STRING, IDM_QUIT as usize, PCWSTR(quit_label.as_ptr())).ok();

                // Position menu at current cursor position
                let mut pt = windows::Win32::Foundation::POINT { x: 0, y: 0 };
                windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut pt).ok();

                // Required so the menu dismisses when clicking outside it
                SetForegroundWindow(self.hwnd).ok();

                TrackPopupMenu(
                    hmenu,
                    TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_RETURNCMD,
                    pt.x,
                    pt.y,
                    0,
                    self.hwnd,
                    None,
                ).ok();

                DestroyMenu(hmenu).ok();
            }
        }
    }

    impl Drop for TrayIcon {
        fn drop(&mut self) {
            let _ = self.hide();
        }
    }

    /// Create a minimal invisible message-only window to receive tray events.
    fn create_message_window() -> Result<HWND> {
        let class_name: Vec<u16> = "WrenflowTray\0".encode_utf16().collect();
        let window_name: Vec<u16> = "Wrenflow\0".encode_utf16().collect();

        unsafe {
            let hinstance = GetModuleHandleW(PCWSTR::null())
                .context("GetModuleHandleW failed")?;

            let wc = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wnd_proc),
                hInstance: hinstance.into(),
                lpszClassName: PCWSTR(class_name.as_ptr()),
                ..Default::default()
            };
            RegisterClassW(&wc); // ignore error if already registered

            let hwnd = CreateWindowExW(
                Default::default(),
                PCWSTR(class_name.as_ptr()),
                PCWSTR(window_name.as_ptr()),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT, CW_USEDEFAULT,
                CW_USEDEFAULT, CW_USEDEFAULT,
                HWND(std::ptr::null_mut()), // HWND_MESSAGE would be ideal but skip for simplicity
                HMENU(std::ptr::null_mut()),
                hinstance,
                None,
            ).context("CreateWindowExW failed")?;

            Ok(hwnd)
        }
    }

    /// Build a `NOTIFYICONDATAW` with the standard application icon.
    fn build_nid(hwnd: HWND, callback_msg: u32) -> NOTIFYICONDATAW {
        let tip: Vec<u16> = {
            let mut v: Vec<u16> = "Wrenflow — hold hotkey to dictate".encode_utf16().collect();
            v.resize(128, 0);
            v
        };
        let mut tip_arr = [0u16; 128];
        tip_arr.copy_from_slice(&tip[..128]);

        let icon = unsafe {
            LoadIconW(None, IDI_APPLICATION).unwrap_or_default()
        };

        let mut nid = NOTIFYICONDATAW {
            cbSize: mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: 1,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
            uCallbackMessage: callback_msg,
            hIcon: icon,
            szTip: tip_arr,
            ..Default::default()
        };
        nid
    }

    /// Minimal window procedure for the tray message window.
    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if msg == WM_DESTROY {
            PostQuitMessage(0);
        }
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

// ---------------------------------------------------------------------------
// Non-Windows stub
// ---------------------------------------------------------------------------

#[cfg(not(target_os = "windows"))]
mod platform {
    use anyhow::Result;
    use tokio::sync::mpsc::UnboundedSender;
    use super::TrayCommand;

    #[allow(dead_code)]
    pub struct TrayIcon;

    #[allow(dead_code)]
    impl TrayIcon {
        pub fn new(_tx: UnboundedSender<TrayCommand>) -> Result<Self> {
            Ok(TrayIcon)
        }

        pub fn show(&mut self) -> Result<()> { Ok(()) }
        pub fn hide(&mut self) -> Result<()> { Ok(()) }
        pub fn pump_messages(&self) {}
    }
}

#[allow(unused_imports)]
pub use platform::TrayIcon;
