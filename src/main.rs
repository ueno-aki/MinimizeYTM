#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod tray;
mod window;

use tray::run_tray_until_double_click;
use window::{WindowInfo, get_window_icon, get_youtube_music_windows, hide_window, show_window};

fn main() {
    let windows: Vec<WindowInfo> = get_youtube_music_windows();

    if windows.is_empty() {
        println!("YouTube Music のウィンドウが見つかりませんでした。");
        return;
    }

    let target_window: &WindowInfo = &windows[0];
    println!(
        "対象ウィンドウ: [PID: {}] {} - {}",
        target_window.process_id, target_window.process_name, target_window.title
    );

    let icon_handle: isize = get_window_icon(target_window.hwnd);

    if !hide_window(target_window.hwnd) {
        println!("YouTube Music の非表示に失敗しました。");
        return;
    }

    println!(
        "YouTube Music を非表示にしました。トレイアイコンをダブルクリックすると復元して終了します。"
    );

    if let Err(error) = run_tray_until_double_click(target_window.hwnd, icon_handle) {
        println!("トレイ処理でエラーが発生しました: {}", error);
        let _: bool = show_window(target_window.hwnd);
    }
}
