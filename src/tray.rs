use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

use crate::icon::load_tray_icon;

static SHOULD_QUIT: AtomicBool = AtomicBool::new(false);
static CACHED_HWND: AtomicIsize = AtomicIsize::new(0);
const WM_TRAYICON: u32 = windows_sys::Win32::UI::WindowsAndMessaging::WM_APP + 1;
const ID_MENU_EXIT: u16 = 1001;

pub fn run_tray() -> Result<(), String> {
    use windows_sys::Win32::UI::Shell::{
        NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, Shell_NotifyIconW,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DispatchMessageW, GWLP_WNDPROC, GetMessageW, MSG, SetWindowLongPtrW,
        TranslateMessage,
    };
    use windows_sys::core::w;

    let class_name: *const u16 = w!("STATIC");
    let window_name: *const u16 = w!("MinimizeYTMHiddenWindow");

    let hwnd: windows_sys::Win32::Foundation::HWND = unsafe {
        CreateWindowExW(
            0,
            class_name,
            window_name,
            0,
            0,
            0,
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null(),
        )
    };
    if hwnd.is_null() {
        return Err("CreateWindowExW failed".to_string());
    }

    #[allow(function_casts_as_integer)]
    unsafe {
        SetWindowLongPtrW(hwnd, GWLP_WNDPROC, tray_window_proc as usize as isize);
    }

    let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    nid.uCallbackMessage = WM_TRAYICON;

    nid.hIcon = load_tray_icon();

    let tip: Vec<u16> = to_wide("MinimizeYTM - YouTube Music toggler");
    for (index, value) in tip.iter().enumerate().take(nid.szTip.len() - 1) {
        nid.szTip[index] = *value;
    }

    let added: i32 = unsafe { Shell_NotifyIconW(NIM_ADD, &nid) };
    if added == 0 {
        return Err("Shell_NotifyIconW(NIM_ADD) failed".to_string());
    }

    let mut msg: MSG = unsafe { std::mem::zeroed() };
    loop {
        let ret: i32 = unsafe { GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) };
        if ret == -1 {
            unsafe {
                Shell_NotifyIconW(NIM_DELETE, &nid);
            }
            return Err("GetMessageW failed".to_string());
        }
        if ret == 0 || SHOULD_QUIT.load(Ordering::SeqCst) {
            break;
        }
        unsafe {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    unsafe {
        Shell_NotifyIconW(NIM_DELETE, &nid);
    }

    Ok(())
}

extern "system" fn tray_window_proc(
    hwnd: windows_sys::Win32::Foundation::HWND,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DefWindowProcW, PostQuitMessage, WM_COMMAND, WM_DESTROY, WM_LBUTTONDBLCLK, WM_RBUTTONUP,
    };

    if msg == WM_TRAYICON {
        let event: u32 = lparam as u32;

        if event == WM_LBUTTONDBLCLK {
            toggle_youtube_music_window();
            return 0;
        }

        if event == WM_RBUTTONUP {
            show_context_menu(hwnd);
            return 0;
        }
    }

    if msg == WM_COMMAND {
        let command_id: u16 = (wparam & 0xFFFF) as u16;
        if command_id == ID_MENU_EXIT {
            handle_exit_command();
            return 0;
        }
    }

    if msg == WM_DESTROY {
        unsafe {
            PostQuitMessage(0);
        }
        return 0;
    }

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

fn toggle_youtube_music_window() {
    let Some(hwnd) = resolve_target_hwnd() else {
        return;
    };

    toggle_window_visibility(hwnd);
    CACHED_HWND.store(hwnd, Ordering::SeqCst);
}

fn handle_exit_command() {
    use crate::window::{is_window_valid, show_window};
    use windows_sys::Win32::UI::WindowsAndMessaging::PostQuitMessage;

    let cached_hwnd: isize = CACHED_HWND.load(Ordering::SeqCst);
    if is_window_valid(cached_hwnd) {
        show_window(cached_hwnd);
    }

    SHOULD_QUIT.store(true, Ordering::SeqCst);
    unsafe {
        PostQuitMessage(0);
    }
}

fn resolve_target_hwnd() -> Option<isize> {
    use crate::window::{WindowInfo, get_youtube_music_windows, is_window_valid};

    // まずはキャッシュを使い、使えないときだけ再検索する。
    let cached_hwnd: isize = CACHED_HWND.load(Ordering::SeqCst);
    if cached_hwnd != 0 && is_window_valid(cached_hwnd) {
        println!("キャッシュされたウィンドウを使用します。");
        return Some(cached_hwnd);
    }

    println!("YouTube Music ウィンドウを検索します...");
    let windows: Vec<WindowInfo> = get_youtube_music_windows();
    if windows.is_empty() {
        println!("YouTube Music のウィンドウが見つかりませんでした。");
        CACHED_HWND.store(0, Ordering::SeqCst);
        return None;
    }

    let target_window: &WindowInfo = &windows[0];
    println!(
        "対象ウィンドウ: {} - {}",
        target_window.process_name, target_window.title
    );
    Some(target_window.hwnd)
}

fn toggle_window_visibility(hwnd: isize) {
    use crate::window::{hide_window, is_window_visible, show_window};

    if is_window_visible(hwnd) {
        println!("YouTube Music を非表示にします。");
        hide_window(hwnd);
    } else {
        println!("YouTube Music を表示します。");
        show_window(hwnd);
    }
}

fn show_context_menu(hwnd: windows_sys::Win32::Foundation::HWND) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        AppendMenuW, CreatePopupMenu, DestroyMenu, GetCursorPos, MF_STRING, SetForegroundWindow,
        TPM_BOTTOMALIGN, TPM_LEFTALIGN, TrackPopupMenu,
    };

    let hmenu: windows_sys::Win32::UI::WindowsAndMessaging::HMENU = unsafe { CreatePopupMenu() };
    if hmenu.is_null() {
        return;
    }

    let exit_text: Vec<u16> = to_wide("終了(&X)");
    unsafe {
        AppendMenuW(hmenu, MF_STRING, ID_MENU_EXIT as usize, exit_text.as_ptr());
    }

    let mut cursor_pos: windows_sys::Win32::Foundation::POINT = unsafe { std::mem::zeroed() };
    unsafe {
        GetCursorPos(&mut cursor_pos);
        SetForegroundWindow(hwnd);
        TrackPopupMenu(
            hmenu,
            TPM_BOTTOMALIGN | TPM_LEFTALIGN,
            cursor_pos.x,
            cursor_pos.y,
            0,
            hwnd,
            std::ptr::null(),
        );
        DestroyMenu(hmenu);
    }
}

fn to_wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}
