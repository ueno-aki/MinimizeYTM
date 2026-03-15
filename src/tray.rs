#[derive(Debug, thiserror::Error)]
pub enum TrayError {
    #[error("Windows API error: {0}")]
    Windows(#[from] windows::core::Error),
    #[error("Failed to add tray icon")]
    NotifyIconAdd,
    #[error("Failed to Set Timer")]
    SetTimer,
}

const WM_TRAY_ICON: u32 = windows::Win32::UI::WindowsAndMessaging::WM_APP + 1;
const ID_HOTKEY_TOGGLE: i32 = 1;
const ID_MENU_EXIT: usize = 1001;
const ID_TIMER: usize = 1;
static SHOULD_QUIT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static CACHED_HWND: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);

pub fn run_tray() -> Result<(), TrayError> {
    use windows::Win32::UI::{
        Input::KeyboardAndMouse::{MOD_WIN, RegisterHotKey, UnregisterHotKey},
        Shell::{
            NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, Shell_NotifyIconW,
        },
        WindowsAndMessaging::{
            CreateWindowExW, DispatchMessageW, GWLP_WNDPROC, GetMessageW, KillTimer, MSG, SetTimer,
            SetWindowLongPtrW, TranslateMessage, WINDOW_EX_STYLE, WINDOW_STYLE,
        },
    };
    use windows::core::{PCWSTR, w};
    let class_name: PCWSTR = w!("STATIC");
    let window_name: PCWSTR = w!("MinimizeYTMHiddenWindow");

    let hwnd: windows::Win32::Foundation::HWND = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class_name,
            window_name,
            WINDOW_STYLE(0),
            0,
            0,
            0,
            0,
            None,
            None,
            None,
            None,
        )
    }?;

    unsafe {
        SetWindowLongPtrW(hwnd, GWLP_WNDPROC, tray_window_proc as *const () as isize);
    }

    let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    nid.uCallbackMessage = WM_TRAY_ICON; // Custom message for tray interactions
    nid.hIcon = crate::tray_icon::load_tray_icon()?;

    let tip_bytes: PCWSTR = w!("MinimizeYTM");
    nid.szTip[..unsafe { tip_bytes.len() }].copy_from_slice(unsafe { tip_bytes.as_wide() });

    if !unsafe { Shell_NotifyIconW(NIM_ADD, &nid) }.as_bool() {
        return Err(TrayError::NotifyIconAdd);
    }

    // Win + Y をグローバルホットキーとして登録する。
    // Win キーとの組み合わせは OS 側で予約されている場合があり、失敗してもトレイ動作は継続する。
    let mut hotkey_registered: bool = true;
    if let Err(e) = unsafe { RegisterHotKey(Some(hwnd), ID_HOTKEY_TOGGLE, MOD_WIN, b'Y' as u32) } {
        hotkey_registered = false;
        println!(
            "Win + Y のホットキー登録に失敗しました。トレイ操作は引き続き利用できます。:{e:?}"
        );
    }

    let ret: usize = unsafe { SetTimer(Some(hwnd), ID_TIMER, 2000, Some(tray_timer_proc)) };
    if ret == 0 {
        return Err(TrayError::SetTimer);
    }

    let mut msg: MSG = unsafe { std::mem::zeroed() };
    loop {
        if !unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
            break;
        }
        if SHOULD_QUIT.load(std::sync::atomic::Ordering::Acquire) {
            break;
        }
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    if hotkey_registered {
        unsafe { UnregisterHotKey(Some(hwnd), ID_HOTKEY_TOGGLE) }?;
    }
    unsafe {
        let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
    }
    unsafe { KillTimer(Some(hwnd), ID_TIMER) }?;

    Ok(())
}

extern "system" fn tray_window_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::UI::WindowsAndMessaging::{
        DefWindowProcW, PostQuitMessage, WM_COMMAND, WM_DESTROY, WM_HOTKEY, WM_LBUTTONDBLCLK,
        WM_RBUTTONUP,
    };
    match msg {
        WM_HOTKEY => {
            if wparam.0 == ID_HOTKEY_TOGGLE as usize {
                if let Err(e) = toggle_target_hwnd() {
                    println!("HOTKEY toggle_target_hwnd error {e:?}.")
                }
                return windows::Win32::Foundation::LRESULT(0);
            }
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        WM_TRAY_ICON => match lparam.0 as u32 {
            WM_LBUTTONDBLCLK => {
                if let Err(e) = toggle_target_hwnd() {
                    println!("TRAY_ICON toggle_target_hwnd error {e:?}.")
                }
                windows::Win32::Foundation::LRESULT(0)
            }
            WM_RBUTTONUP => {
                if let Err(e) = show_context_menu(hwnd) {
                    println!("TRAY_ICON context_menu error {e:?}.");
                };
                windows::Win32::Foundation::LRESULT(0)
            }
            _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
        },
        WM_COMMAND => {
            let command_id: usize = wparam.0 & 0xFFFF;
            if command_id == ID_MENU_EXIT {
                let cached_hwnd = windows::Win32::Foundation::HWND(
                    CACHED_HWND.load(std::sync::atomic::Ordering::Relaxed) as *mut std::ffi::c_void,
                );
                if crate::window::is_window_valid(cached_hwnd) {
                    crate::window::show_window(cached_hwnd);
                }
                SHOULD_QUIT.store(true, std::sync::atomic::Ordering::Release);
                unsafe {
                    PostQuitMessage(0);
                }
                windows::Win32::Foundation::LRESULT(0)
            } else {
                unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
            }
        }
        WM_DESTROY => {
            unsafe {
                PostQuitMessage(0);
            };
            windows::Win32::Foundation::LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

fn toggle_target_hwnd() -> Result<(), windows::core::Error> {
    match resolve_target_hwnd() {
        Ok(Some(hwnd)) => {
            if crate::window::is_window_visible(hwnd) {
                println!("YouTube Music を非表示にします。");
                crate::window::hide_window(hwnd);
            } else {
                println!("YouTube Music を表示します。");
                crate::window::show_window(hwnd);
            }
            Ok(())
        }
        Ok(None) => Ok(()),
        Err(e) => Err(e),
    }
}

fn resolve_target_hwnd() -> Result<Option<windows::Win32::Foundation::HWND>, windows::core::Error> {
    let cached_hwnd = windows::Win32::Foundation::HWND(
        CACHED_HWND.load(std::sync::atomic::Ordering::Relaxed) as *mut std::ffi::c_void,
    );
    if crate::window::is_window_valid(cached_hwnd) {
        println!("キャッシュされたウィンドウを使用します。");
        return Ok(Some(cached_hwnd));
    }

    println!("YouTube Music ウィンドウを検索します...");
    let window_infos: Vec<crate::window::WindowInfo> = get_ytm_window_infos()?;
    if window_infos.is_empty() {
        println!("YouTube Music のウィンドウが見つかりませんでした。");
        CACHED_HWND.store(0, std::sync::atomic::Ordering::Relaxed);
        Ok(None)
    } else {
        let target_window: &crate::window::WindowInfo = &window_infos[0];
        println!("対象ウィンドウ: {:?}", target_window);
        CACHED_HWND.store(
            target_window.hwnd.0 as isize,
            std::sync::atomic::Ordering::Relaxed,
        );
        Ok(Some(target_window.hwnd))
    }
}

fn get_ytm_window_infos() -> Result<Vec<crate::window::WindowInfo>, windows::core::Error> {
    let ret: Vec<crate::window::WindowInfo> = crate::window::get_default_windows()?
        .into_iter()
        .filter(|info| info.app_user_model_id == crate::YTM_APP_ID)
        .collect();
    Ok(ret)
}

fn show_context_menu(hwnd: windows::Win32::Foundation::HWND) -> Result<(), windows::core::Error> {
    use windows::Win32::UI::WindowsAndMessaging::{
        AppendMenuW, CreatePopupMenu, DestroyMenu, GetCursorPos, MF_STRING, SetForegroundWindow,
        TPM_BOTTOMALIGN, TPM_LEFTALIGN, TrackPopupMenu,
    };

    let exit_text: windows::core::HSTRING = "終了(&X)".into();
    let hmenu: windows::Win32::UI::WindowsAndMessaging::HMENU = unsafe { CreatePopupMenu() }?;
    unsafe {
        AppendMenuW(
            hmenu,
            MF_STRING,
            ID_MENU_EXIT,
            windows::core::PCWSTR::from_raw(exit_text.as_ptr()),
        )
    }?;

    let mut cursor_pos: windows::Win32::Foundation::POINT = unsafe { std::mem::zeroed() };
    unsafe { GetCursorPos(&mut cursor_pos) }?;
    unsafe {
        let _ = SetForegroundWindow(hwnd);
    };
    unsafe {
        let _ = TrackPopupMenu(
            hmenu,
            TPM_BOTTOMALIGN | TPM_LEFTALIGN,
            cursor_pos.x,
            cursor_pos.y,
            None,
            hwnd,
            None,
        );
    };
    unsafe { DestroyMenu(hmenu) }?;
    Ok(())
}

pub static PREV_SESSION_INFO: std::sync::LazyLock<std::sync::Mutex<(String, String)>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(Default::default()));

extern "system" fn tray_timer_proc(
    _hwnd: windows::Win32::Foundation::HWND,
    _message: u32,
    _id: usize,
    _dw_time: u32,
) {
    match is_music_changed() {
        Ok(true) => {}
        Ok(false) => return,
        Err(e) => {
            println!("is_music_changed is failed {e:?}");
            return;
        }
    }
    std::thread::sleep(std::time::Duration::from_millis(1000));
    if let Err(e) = send_toast() {
        println!("send_toast is failed {e:?}");
    }
}

fn is_music_changed() -> Result<bool, windows::core::Error> {
    let sessions: Vec<crate::audio::MediaSessionInfo> = crate::audio::get_current_media_sessions()?
        .into_iter()
        .filter(|session| session.source_app_id == crate::YTM_APP_ID)
        .collect();
    if sessions.is_empty() {
        Ok(false)
    } else {
        let mut cached_info = PREV_SESSION_INFO.lock().unwrap();
        if sessions[0].title == cached_info.0 && sessions[0].artist == cached_info.1 {
            Ok(false)
        } else {
            *cached_info = (sessions[0].title.clone(), sessions[0].artist.clone());
            Ok(true)
        }
    }
}

fn send_toast() -> Result<(), windows::core::Error> {
    use std::{
        fs::File,
        io::{BufWriter, copy},
    };
    let sessions: Vec<crate::audio::MediaSessionInfo> = crate::audio::get_current_media_sessions()?
        .into_iter()
        .filter(|session| session.source_app_id == crate::YTM_APP_ID)
        .collect();
    if sessions.is_empty() {
        return Ok(());
    }
    let mut thumnail: &[u8] = &sessions[0].get_thumnail()?[..];
    let file_path: std::path::PathBuf = std::env::temp_dir().join("ytm_thumbnail.png");
    let mut writer: BufWriter<File> = BufWriter::new(File::create(&file_path).unwrap());
    if let Err(e) = copy(&mut thumnail, &mut writer) {
        println!("Failed to copy into bufWriter :{e:?}")
    }
    crate::toast::toast_show(
        &sessions[0].title,
        &sessions[0].artist,
        &format!("{}", file_path.display()),
        "Chrome._crx_cinhimbnkkghhklpknlkffjgod",
    )?;
    Ok(())
}
