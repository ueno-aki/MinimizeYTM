use std::sync::atomic::{AtomicIsize, Ordering};

static TARGET_HWND: AtomicIsize = AtomicIsize::new(0);
const WM_TRAYICON: u32 = windows_sys::Win32::UI::WindowsAndMessaging::WM_APP + 1;

pub fn run_tray_until_double_click(target_hwnd: isize, icon_handle: isize) -> Result<(), String> {
    use windows_sys::Win32::UI::Shell::{
        NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, Shell_NotifyIconW,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DispatchMessageW, GWLP_WNDPROC, GetMessageW, IDI_APPLICATION, LoadIconW,
        MSG, SetWindowLongPtrW, TranslateMessage,
    };
    use windows_sys::core::w;

    TARGET_HWND.store(target_hwnd, Ordering::SeqCst);

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
    let tray_icon: windows_sys::Win32::UI::WindowsAndMessaging::HICON = if icon_handle != 0 {
        icon_handle as windows_sys::Win32::UI::WindowsAndMessaging::HICON
    } else {
        unsafe { LoadIconW(std::ptr::null_mut(), IDI_APPLICATION) }
    };
    nid.hIcon = tray_icon;

    let tip: Vec<u16> = to_wide("Double-click to restore YouTube Music");
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
        if ret == 0 {
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
        DefWindowProcW, PostQuitMessage, SW_RESTORE, SW_SHOW, ShowWindow, WM_DESTROY,
        WM_LBUTTONDBLCLK,
    };

    if msg == WM_TRAYICON && (lparam as u32) == WM_LBUTTONDBLCLK {
        let target_hwnd_raw: isize = TARGET_HWND.load(Ordering::SeqCst);
        let target_hwnd: windows_sys::Win32::Foundation::HWND =
            target_hwnd_raw as windows_sys::Win32::Foundation::HWND;

        if !target_hwnd.is_null() {
            unsafe {
                ShowWindow(target_hwnd, SW_SHOW);
                ShowWindow(target_hwnd, SW_RESTORE);
            }
        }

        unsafe {
            PostQuitMessage(0);
        }
        return 0;
    }

    if msg == WM_DESTROY {
        unsafe {
            PostQuitMessage(0);
        }
        return 0;
    }

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

fn to_wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}
