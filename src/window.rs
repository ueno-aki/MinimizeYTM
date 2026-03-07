#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub hwnd: isize,
    pub title: String,
    pub process_name: String,
}

pub fn get_youtube_music_windows() -> Vec<WindowInfo> {
    use regex::Regex;

    // 既知のタイトル形式を許容:
    // 1) YouTube Music
    // 2) YouTube Music - $曲名 | YouTube Music
    // 3) YouTube Music - $曲名 - $洋名 | YouTube Music
    let pattern: Regex = Regex::new(
        r"^(YouTube Music|YouTube Music - .+ \| YouTube Music|YouTube Music - .+ - .+ \| YouTube Music)$",
    )
    .unwrap();

    get_open_windows()
        .into_iter()
        .filter(|w| w.process_name == "chrome.exe" && pattern.is_match(&w.title))
        .collect()
}

fn get_open_windows() -> Vec<WindowInfo> {
    use windows_sys::Win32::Foundation::LPARAM;
    use windows_sys::Win32::UI::WindowsAndMessaging::EnumWindows;

    let mut windows: Vec<WindowInfo> = Vec::new();

    unsafe {
        let lparam: LPARAM = (&mut windows as *mut Vec<WindowInfo>) as LPARAM;
        EnumWindows(Some(enum_windows_proc), lparam);
    }

    windows
}

unsafe extern "system" fn enum_windows_proc(
    hwnd: windows_sys::Win32::Foundation::HWND,
    lparam: windows_sys::Win32::Foundation::LPARAM,
) -> i32 {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GW_OWNER, GWL_EXSTYLE, GetWindow, GetWindowLongW, GetWindowTextLengthW, GetWindowTextW,
        IsWindowVisible, WS_EX_TOOLWINDOW,
    };

    if unsafe { IsWindowVisible(hwnd) } == 0 {
        return 1;
    }

    let ex_style: u32 = unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) as u32 };
    let has_tool_window: bool = (ex_style & WS_EX_TOOLWINDOW) != 0;
    let has_owner: bool = !unsafe { GetWindow(hwnd, GW_OWNER) }.is_null();

    if has_owner || has_tool_window {
        return 1;
    }

    let title_len: i32 = unsafe { GetWindowTextLengthW(hwnd) };
    if title_len <= 0 {
        return 1;
    }

    let mut buffer: Vec<u16> = vec![0; (title_len + 1) as usize];
    let copied_len: i32 = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), title_len + 1) };
    if copied_len <= 0 {
        return 1;
    }

    let title: String = String::from_utf16_lossy(&buffer[..copied_len as usize]);

    let mut process_id: u32 = 0;
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
        GetWindowThreadProcessId(hwnd, &mut process_id as *mut u32);
    }

    let process_name: String =
        get_window_process_name(hwnd).unwrap_or_else(|| "unknown".to_string());

    let window_info: WindowInfo = WindowInfo {
        hwnd: hwnd as isize,
        title,
        process_name,
    };

    let windows_ptr: *mut Vec<WindowInfo> = lparam as *mut Vec<WindowInfo>;
    if windows_ptr.is_null() {
        return 1;
    }

    unsafe {
        (*windows_ptr).push(window_info);
    }

    1
}

fn get_window_process_name(hwnd: windows_sys::Win32::Foundation::HWND) -> Option<String> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

    let mut process_id: u32 = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, &mut process_id as *mut u32);
    }
    if process_id == 0 {
        return None;
    }

    let process_handle: windows_sys::Win32::Foundation::HANDLE =
        unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id) };
    if process_handle.is_null() {
        return None;
    }

    let mut buffer: Vec<u16> = vec![0; 1024];
    let mut size: u32 = buffer.len() as u32;
    let ok: i32 =
        unsafe { QueryFullProcessImageNameW(process_handle, 0, buffer.as_mut_ptr(), &mut size) };

    unsafe {
        CloseHandle(process_handle);
    }

    if ok == 0 || size == 0 {
        return None;
    }

    let full_path: String = String::from_utf16_lossy(&buffer[..size as usize]);
    let process_name: String = full_path
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or(full_path.as_str())
        .to_ascii_lowercase();

    Some(process_name)
}

pub fn hide_window(hwnd: isize) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{SW_HIDE, ShowWindow};

    unsafe { ShowWindow(hwnd as windows_sys::Win32::Foundation::HWND, SW_HIDE) };
    true
}

pub fn show_window(hwnd: isize) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{SW_RESTORE, SW_SHOW, ShowWindow};

    unsafe {
        ShowWindow(hwnd as windows_sys::Win32::Foundation::HWND, SW_SHOW);
        ShowWindow(hwnd as windows_sys::Win32::Foundation::HWND, SW_RESTORE);
    };
    true
}

pub fn is_window_visible(hwnd: isize) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::IsWindowVisible;

    unsafe { IsWindowVisible(hwnd as windows_sys::Win32::Foundation::HWND) != 0 }
}

pub fn is_window_valid(hwnd: isize) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::IsWindow;

    if hwnd == 0 {
        return false;
    }
    unsafe { IsWindow(hwnd as windows_sys::Win32::Foundation::HWND) != 0 }
}
