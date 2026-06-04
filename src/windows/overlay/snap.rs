//! Windows 11 snap-layout support for the auto-injected caption controls.
//!
//! A small transparent hit-test-only child window is placed exactly over the
//! Maximize button and returns `HTMAXBUTTON` from `WM_NCHITTEST`, which is the
//! OS-supported way to make the snap-layout flyout appear on hover. Because the
//! overlay sits on top of the DOM button, it can't see DOM hover/click, so it
//! bridges those back to the webview as events. The frontend 
//! performs the maximize action itself (via `window_command`) on the `click` event.
//!
//! Position is derived natively from the fixed caption geometry (46px buttons,
//! maximize is second from the right) + DPI, so no rect needs to be reported
//! from JS. Adapted from `tauri-plugin-frame`'s proven approach.

use std::{collections::HashMap, sync::Mutex};

use tauri::{Emitter, EventTarget, Runtime, WebviewWindow};
use windows_sys::Win32::{
    Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
    Graphics::Gdi::{GetStockObject, HBRUSH, NULL_BRUSH},
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        HiDpi::GetDpiForWindow,
        Input::KeyboardAndMouse::{TrackMouseEvent, TME_LEAVE, TME_NONCLIENT, TRACKMOUSEEVENT},
        Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
        WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect, RegisterClassExW,
            SetWindowPos, CS_HREDRAW, CS_VREDRAW, HTMAXBUTTON, HWND_TOP, SWP_ASYNCWINDOWPOS,
            SWP_SHOWWINDOW, WM_CLOSE, WM_DPICHANGED, WM_NCHITTEST, WM_NCLBUTTONDOWN,
            WM_NCLBUTTONUP, WM_NCMOUSELEAVE, WM_NCMOUSEMOVE, WM_SIZE, WNDCLASSEXW, WS_CHILD,
            WS_CLIPSIBLINGS, WS_OVERLAPPED, WS_VISIBLE,
        },
    },
};

/// `"TboSnapOverlay\0"` as UTF-16.
const CLASS: &[u16] = &[
    0x54, 0x62, 0x6f, 0x53, 0x6e, 0x61, 0x70, 0x4f, 0x76, 0x65, 0x72, 0x6c, 0x61, 0x79, 0x00,
];
const SUBCLASS_ID: usize = 0x7462_6f73_6e70;

/// Each caption button is 46px wide; maximize has one button (close) to its right.
const BUTTON_WIDTH: u32 = 46;
const RIGHT_INDEX: u32 = 1;

const EVENT_ENTER: &str = "window-controls://snap-enter";
const EVENT_LEAVE: &str = "window-controls://snap-leave";
const EVENT_DOWN: &str = "window-controls://snap-down";
const EVENT_UP: &str = "window-controls://snap-up";
const EVENT_CLICK: &str = "window-controls://snap-click";

static OVERLAYS: Mutex<Option<HashMap<isize, SnapState>>> = Mutex::new(None);

struct SnapState {
    overlay: HWND,
    titlebar_height: u32,
    hovering: bool,
    pressing: bool,
    emit: Box<dyn Fn(&'static str) + Send>,
}

// SAFETY: HWNDs are only touched on the UI thread; the map is guarded by a Mutex.
unsafe impl Send for SnapState {}

fn with_states<T>(f: impl FnOnce(&mut HashMap<isize, SnapState>) -> T) -> T {
    let mut guard = OVERLAYS.lock().expect("snap state poisoned");
    f(guard.get_or_insert_with(HashMap::new))
}

/// Installs (or reinstalls) the snap overlay on `window`'s maximize button.
pub(crate) fn install<R: Runtime>(window: &WebviewWindow<R>, titlebar_height: u32) -> tauri::Result<()> {
    let hwnd = window.hwnd()?.0 as isize;
    let emitter = window.clone();
    let label = window.label().to_string();

    window.run_on_main_thread(move || unsafe {
        install_hwnd(
            hwnd,
            titlebar_height,
            Box::new(move |event| {
                // Scope to this window's webview only. A bare `emit` broadcasts
                // app-wide, so every window's caption runtime would react to one
                // window's hover/press — `emit_to` keeps it to the source window.
                let _ = emitter.emit_to(EventTarget::webview_window(label.clone()), event, ());
            }),
        );
    })?;

    Ok(())
}

unsafe fn install_hwnd(hwnd: isize, titlebar_height: u32, emit: Box<dyn Fn(&'static str) + Send>) {
    register_class();

    let overlay = CreateWindowExW(
        0,
        CLASS.as_ptr(),
        CLASS.as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS | WS_OVERLAPPED,
        0,
        0,
        0,
        0,
        hwnd as HWND,
        std::ptr::null_mut(),
        module_instance(),
        std::ptr::null(),
    );

    if overlay.is_null() {
        return;
    }

    with_states(|states| {
        if let Some(old) = states.remove(&hwnd) {
            DestroyWindow(old.overlay);
        }
        states.insert(
            hwnd,
            SnapState {
                overlay,
                titlebar_height,
                hovering: false,
                pressing: false,
                emit,
            },
        );
    });

    SetWindowSubclass(hwnd as HWND, Some(parent_subclass_proc), SUBCLASS_ID, 0);
    update_overlay_position(hwnd as HWND);
}

unsafe fn register_class() {
    let class = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(overlay_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: module_instance(),
        hIcon: std::ptr::null_mut(),
        hCursor: std::ptr::null_mut(),
        hbrBackground: GetStockObject(NULL_BRUSH) as HBRUSH,
        lpszMenuName: std::ptr::null(),
        lpszClassName: CLASS.as_ptr(),
        hIconSm: std::ptr::null_mut(),
    };
    RegisterClassExW(&class);
}

unsafe fn module_instance() -> HINSTANCE {
    GetModuleHandleW(std::ptr::null()) as HINSTANCE
}

unsafe fn update_overlay_position(hwnd: HWND) {
    with_states(|states| {
        let Some(state) = states.get(&(hwnd as isize)) else {
            return;
        };

        let mut rect = std::mem::zeroed();
        if GetClientRect(hwnd, &mut rect) == 0 {
            return;
        }

        let dpi = GetDpiForWindow(hwnd) as u64;
        let button_width = scaled(BUTTON_WIDTH, dpi).max(1);
        let height = scaled(state.titlebar_height, dpi).max(1);
        let x = rect.right - button_width * (RIGHT_INDEX as i32 + 1);

        SetWindowPos(
            state.overlay,
            HWND_TOP,
            x,
            0,
            button_width,
            height,
            SWP_ASYNCWINDOWPOS | SWP_SHOWWINDOW,
        );
    });
}

fn scaled(value: u32, dpi: u64) -> i32 {
    ((value as u64 * dpi + 48) / 96) as i32
}

unsafe fn remove(hwnd: HWND) {
    RemoveWindowSubclass(hwnd, Some(parent_subclass_proc), SUBCLASS_ID);
    with_states(|states| {
        if let Some(state) = states.remove(&(hwnd as isize)) {
            DestroyWindow(state.overlay);
        }
    });
}

unsafe fn emit(parent: HWND, event: &'static str) {
    with_states(|states| {
        if let Some(state) = states.get(&(parent as isize)) {
            (state.emit)(event);
        }
    });
}

unsafe fn parent_for_overlay(overlay: HWND) -> Option<HWND> {
    with_states(|states| {
        states
            .iter()
            .find_map(|(parent, state)| (state.overlay == overlay).then_some(*parent as HWND))
    })
}

unsafe extern "system" fn parent_subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    _ref: usize,
) -> LRESULT {
    match msg {
        WM_SIZE | WM_DPICHANGED => update_overlay_position(hwnd),
        WM_CLOSE => remove(hwnd),
        _ => {}
    }
    DefSubclassProc(hwnd, msg, wparam, lparam)
}

unsafe extern "system" fn overlay_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_NCHITTEST => return HTMAXBUTTON as LRESULT,
        WM_NCMOUSEMOVE => {
            if let Some(parent) = parent_for_overlay(hwnd) {
                let entered = with_states(|states| {
                    states.get_mut(&(parent as isize)).map_or(false, |state| {
                        let was = state.hovering;
                        state.hovering = true;
                        !was
                    })
                });
                if entered {
                    emit(parent, EVENT_ENTER);
                }

                let mut track = TRACKMOUSEEVENT {
                    cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags: TME_LEAVE | TME_NONCLIENT,
                    hwndTrack: hwnd,
                    dwHoverTime: 0,
                };
                TrackMouseEvent(&mut track);
            }
            return 0;
        }
        WM_NCMOUSELEAVE => {
            if let Some(parent) = parent_for_overlay(hwnd) {
                with_states(|states| {
                    if let Some(state) = states.get_mut(&(parent as isize)) {
                        state.hovering = false;
                        state.pressing = false;
                    }
                });
                emit(parent, EVENT_LEAVE);
            }
            return 0;
        }
        WM_NCLBUTTONDOWN => {
            if let Some(parent) = parent_for_overlay(hwnd) {
                with_states(|states| {
                    if let Some(state) = states.get_mut(&(parent as isize)) {
                        state.pressing = true;
                    }
                });
                emit(parent, EVENT_DOWN);
            }
            return 0;
        }
        WM_NCLBUTTONUP => {
            if let Some(parent) = parent_for_overlay(hwnd) {
                let click = with_states(|states| {
                    states.get_mut(&(parent as isize)).map_or(false, |state| {
                        let click = state.pressing;
                        state.pressing = false;
                        click
                    })
                });
                emit(parent, EVENT_UP);
                if click {
                    emit(parent, EVENT_CLICK);
                }
            }
            return 0;
        }
        _ => {}
    }

    DefWindowProcW(hwnd, msg, wparam, lparam)
}
