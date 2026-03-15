#[derive(Debug)]
pub struct WindowInfo {
    pub hwnd: windows::Win32::Foundation::HWND,
    pub title: String,
    pub process_id: u32,
    pub app_user_model_id: String,
}

pub fn get_default_windows() -> Result<Vec<WindowInfo>, windows::core::Error> {
    use windows::Win32::Foundation::LPARAM;
    use windows::Win32::UI::WindowsAndMessaging::EnumWindows;
    let mut window_infos: Vec<WindowInfo> = Vec::new();
    unsafe {
        let lparam: LPARAM = LPARAM((&mut window_infos as *mut Vec<WindowInfo>) as isize);
        EnumWindows(Some(enum_windows_proc), lparam)?;
    }
    Ok(window_infos)
}

unsafe extern "system" fn enum_windows_proc(
    hwnd: windows::Win32::Foundation::HWND,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::core::BOOL {
    use windows::Win32::UI::WindowsAndMessaging::{
        GW_OWNER, GWL_EXSTYLE, GetWindow, GetWindowLongW, GetWindowTextLengthW, GetWindowTextW,
        GetWindowThreadProcessId, WS_EX_TOOLWINDOW,
    };

    let ex_style: u32 = unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) as u32 };
    let has_tool_window: bool = (ex_style & WS_EX_TOOLWINDOW.0) != 0;
    let has_owner: bool = unsafe { GetWindow(hwnd, GW_OWNER) }.is_ok();

    if has_owner || has_tool_window {
        return true.into();
    }

    let title_len: i32 = unsafe { GetWindowTextLengthW(hwnd) };
    if title_len <= 0 {
        return true.into();
    }

    let mut buffer: Vec<u16> = vec![0; (title_len + 1) as usize];
    let copied_len: i32 = unsafe { GetWindowTextW(hwnd, &mut buffer) };
    if copied_len <= 0 {
        return true.into();
    }
    let title: String = String::from_utf16_lossy(&buffer[..copied_len as usize]);

    let mut process_id: u32 = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut process_id as *mut u32));
    }

    let app_user_model_id: String = unsafe {
        use windows::Win32::Storage::EnhancedStorage::PKEY_AppUserModel_ID;
        use windows::Win32::System::Com::StructuredStorage::PropVariantToBSTR;
        use windows::Win32::UI::Shell::PropertiesSystem::{
            IPropertyStore, SHGetPropertyStoreForWindow,
        };
        SHGetPropertyStoreForWindow::<IPropertyStore>(hwnd)
            .and_then(|prop| prop.GetValue(&PKEY_AppUserModel_ID))
            .and_then(|id| PropVariantToBSTR(&id))
            .map(|b| b.to_string())
            .unwrap_or_default()
    };

    let window_info: WindowInfo = WindowInfo {
        hwnd,
        title,
        process_id,
        app_user_model_id,
    };

    let window_infos_ptr: *mut Vec<WindowInfo> = lparam.0 as *mut Vec<WindowInfo>;
    if window_infos_ptr.is_null() {
        return true.into();
    }
    unsafe {
        (*window_infos_ptr).push(window_info);
    }

    true.into()
}

pub fn hide_window(hwnd: windows::Win32::Foundation::HWND) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{SW_HIDE, ShowWindow};

    unsafe { ShowWindow(hwnd, SW_HIDE) }.as_bool()
}

pub fn show_window(hwnd: windows::Win32::Foundation::HWND) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{SW_RESTORE, ShowWindow};

    unsafe { ShowWindow(hwnd, SW_RESTORE) }.as_bool()
    // unsafe { ShowWindow(hwnd, SW_SHOW) }.as_bool()
}

pub fn is_window_visible(hwnd: windows::Win32::Foundation::HWND) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::IsWindowVisible;

    unsafe { IsWindowVisible(hwnd) }.as_bool()
}

pub fn is_window_valid(hwnd: windows::Win32::Foundation::HWND) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::IsWindow;

    if hwnd.is_invalid() {
        return false;
    }
    unsafe { IsWindow(Some(hwnd)) }.as_bool()
}
