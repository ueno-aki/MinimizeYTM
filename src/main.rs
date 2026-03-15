#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod toast;
mod tray;
mod tray_icon;
mod window;

const YTM_APP_ID: &str = "Chrome._crx_cinhimbnkkghhklpknlkffjgod";

fn main() {
    println!("MinimizeYTM を起動しました。タスクトレイに常駐します。");
    println!("トレイアイコンをダブルクリックすると YouTube Music の表示/非表示を切り替えます。");
    println!("Win + Y でも YouTube Music の表示/非表示を切り替えられます。\n");

    if let Err(error) = tray::run_tray() {
        println!("トレイ処理でエラーが発生しました: {}", error);
    }
}
