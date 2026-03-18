//! Text insertion into the focused UI element.
//!
//! Strategy (tried in order):
//!  1. UI Automation `ValuePattern.SetValue` — sets the value of the focused
//!     control directly, without touching the clipboard.  Works for most text
//!     fields (edit controls, browser address bars, etc.).
//!  2. Clipboard + Ctrl+V fallback — writes the text to the clipboard and
//!     simulates Ctrl+V.  Works universally but pollutes the clipboard.
//!
//! Platform guard: the real implementation is compiled only on Windows;
//! a no-op stub is provided for `cargo check` on other hosts.

// ---------------------------------------------------------------------------
// Windows real implementation
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
mod platform {
    use anyhow::{Context, Result};
    use log::{debug, warn};
    use windows::core::{Interface, BSTR, HSTRING};
    use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED};
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
    };
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
    use windows::Win32::UI::Accessibility::{
        CUIAutomation, IUIAutomation, IUIAutomationElement, IUIAutomationValuePattern,
        UIA_ValuePatternId,
    };
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_CONTROL, VK_V,
    };
    use windows::Win32::UI::WindowsAndMessaging::CF_UNICODETEXT;
    use windows::Win32::Foundation::HWND;

    pub struct TextInserter {
        automation: Option<IUIAutomation>,
    }

    impl TextInserter {
        pub fn new() -> Self {
            // Initialise COM on this thread (single-threaded apartment).
            // Ignore the error — COM may already be initialised.
            unsafe {
                let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            }

            let automation = unsafe {
                CoCreateInstance::<_, IUIAutomation>(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
                    .ok()
            };
            if automation.is_none() {
                warn!("UI Automation not available — will use clipboard fallback only");
            }

            TextInserter { automation }
        }

        /// Insert `text` into the currently focused UI element.
        pub fn insert(&self, text: &str) -> Result<()> {
            if let Some(ref uia) = self.automation {
                match self.try_value_pattern(uia, text) {
                    Ok(()) => {
                        debug!("text inserted via ValuePattern");
                        return Ok(());
                    }
                    Err(e) => {
                        warn!("ValuePattern failed: {e} — falling back to clipboard");
                    }
                }
            }
            self.clipboard_paste(text)
        }

        // ------------------------------------------------------------------
        // Strategy 1: UI Automation ValuePattern.SetValue
        // ------------------------------------------------------------------

        fn try_value_pattern(&self, uia: &IUIAutomation, text: &str) -> Result<()> {
            let focused: IUIAutomationElement = unsafe {
                uia.GetFocusedElement()
                    .context("GetFocusedElement failed")?
            };

            // Check if the element supports ValuePattern (pattern ID 10002)
            let pattern = unsafe {
                focused
                    .GetCurrentPattern(UIA_ValuePatternId)
                    .context("GetCurrentPattern failed")?
            };

            let value_pattern: IUIAutomationValuePattern = pattern
                .cast::<IUIAutomationValuePattern>()
                .context("element does not support IUIAutomationValuePattern")?;

            let bstr = BSTR::from(text);
            unsafe {
                value_pattern
                    .SetValue(&bstr)
                    .context("ValuePattern.SetValue failed")?;
            }
            Ok(())
        }

        // ------------------------------------------------------------------
        // Strategy 2: Clipboard + Ctrl+V
        // ------------------------------------------------------------------

        fn clipboard_paste(&self, text: &str) -> Result<()> {
            self.set_clipboard_text(text)?;
            self.send_ctrl_v();
            Ok(())
        }

        fn set_clipboard_text(&self, text: &str) -> Result<()> {
            // Encode as UTF-16 with null terminator
            let mut utf16: Vec<u16> = text.encode_utf16().collect();
            utf16.push(0);
            let byte_len = utf16.len() * std::mem::size_of::<u16>();

            unsafe {
                OpenClipboard(HWND(std::ptr::null_mut()))
                    .context("OpenClipboard failed")?;
                EmptyClipboard().context("EmptyClipboard failed")?;

                let hmem = GlobalAlloc(GMEM_MOVEABLE, byte_len)
                    .context("GlobalAlloc failed")?;
                let ptr = GlobalLock(hmem);
                if ptr.is_null() {
                    CloseClipboard().ok();
                    anyhow::bail!("GlobalLock failed");
                }
                std::ptr::copy_nonoverlapping(utf16.as_ptr() as *const u8, ptr as *mut u8, byte_len);
                let _ = GlobalUnlock(hmem);

                SetClipboardData(CF_UNICODETEXT.0 as u32, windows::Win32::Foundation::HANDLE(hmem.0))
                    .context("SetClipboardData failed")?;

                CloseClipboard().context("CloseClipboard failed")?;
            }
            Ok(())
        }

        fn send_ctrl_v(&self) {
            // Press Ctrl, press V, release V, release Ctrl
            let inputs: [INPUT; 4] = [
                make_key_input(VK_CONTROL.0, false),
                make_key_input(VK_V.0, false),
                make_key_input(VK_V.0, true),
                make_key_input(VK_CONTROL.0, true),
            ];
            unsafe {
                SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            }
        }
    }

    fn make_key_input(vk: u16, key_up: bool) -> INPUT {
        let mut flags = windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0);
        if key_up {
            flags |= KEYEVENTF_KEYUP;
        }
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(vk),
                    wScan: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Context capture via UI Automation element tree
// ---------------------------------------------------------------------------
//
// Walks the UI Automation tree around the focused element to collect
// surrounding text context (window title, control name, current value).
// This mirrors the macOS ScreenCaptureKit context capture in spirit.

#[cfg(target_os = "windows")]
pub mod context_capture {
    use anyhow::Result;
    use log::debug;
    use windows::core::Interface;
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
    use windows::Win32::UI::Accessibility::{
        CUIAutomation, IUIAutomation, IUIAutomationElement,
        UIA_NamePropertyId, UIA_ValueValuePropertyId, UIA_ClassNamePropertyId,
    };

    /// Minimal context gathered from the UI around the focused element.
    #[derive(Debug, Default)]
    pub struct FocusContext {
        pub window_title: String,
        pub control_name: String,
        pub control_class: String,
        pub current_value: String,
    }

    /// Capture context from the currently focused UI element.
    pub fn capture_focus_context() -> Result<FocusContext> {
        let automation: IUIAutomation = unsafe {
            CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)?
        };

        let focused: IUIAutomationElement = unsafe { automation.GetFocusedElement()? };

        let control_name = get_string_property(&focused, UIA_NamePropertyId.0)?;
        let control_class = get_string_property(&focused, UIA_ClassNamePropertyId.0)?;
        let current_value = get_string_property(&focused, UIA_ValueValuePropertyId.0)?;

        // Walk up to find the top-level window title
        let window_title = find_window_title(&automation, &focused)?;

        debug!(
            "context: window={window_title:?} control={control_name:?} \
             class={control_class:?} value_len={}",
            current_value.len()
        );

        Ok(FocusContext {
            window_title,
            control_name,
            control_class,
            current_value,
        })
    }

    fn get_string_property(element: &IUIAutomationElement, property_id: i32) -> Result<String> {
        use windows::Win32::UI::Accessibility::PROPERTYID;
        let variant = unsafe {
            element.GetCurrentPropertyValue(PROPERTYID(property_id))?
        };
        // VARIANT → BSTR extraction
        let bstr = unsafe { variant.Anonymous.Anonymous.Anonymous.bstrVal.to_string() };
        Ok(bstr)
    }

    fn find_window_title(
        automation: &IUIAutomation,
        start: &IUIAutomationElement,
    ) -> Result<String> {
        use windows::Win32::UI::Accessibility::{TreeScope_Ancestors, UIA_WindowControlTypeId};
        // Walk the ancestor tree to find the nearest Window control type
        let condition = unsafe {
            automation.CreatePropertyCondition(
                windows::Win32::UI::Accessibility::UIA_ControlTypePropertyId,
                &windows::Win32::System::Variant::VARIANT::from(UIA_WindowControlTypeId.0),
            )?
        };
        if let Ok(ancestor) = unsafe {
            start.FindFirst(TreeScope_Ancestors, &condition)
        } {
            get_string_property(&ancestor, UIA_NamePropertyId.0)
        } else {
            Ok(String::new())
        }
    }
}

// ---------------------------------------------------------------------------
// Non-Windows stub
// ---------------------------------------------------------------------------

#[cfg(not(target_os = "windows"))]
mod platform {
    use anyhow::Result;

    #[allow(dead_code)]
    pub struct TextInserter;

    #[allow(dead_code)]
    impl TextInserter {
        pub fn new() -> Self { TextInserter }

        pub fn insert(&self, _text: &str) -> Result<()> {
            Ok(())
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub mod context_capture {
    use anyhow::Result;

    #[derive(Debug, Default)]
    #[allow(dead_code)]
    pub struct FocusContext {
        pub window_title: String,
        pub control_name: String,
        pub control_class: String,
        pub current_value: String,
    }

    #[allow(dead_code)]
    pub fn capture_focus_context() -> Result<FocusContext> {
        Ok(FocusContext::default())
    }
}

#[allow(unused_imports)]
pub use platform::TextInserter;
