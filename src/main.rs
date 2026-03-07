#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod icon;
mod tray;
mod window;

use tray::run_tray;

fn main() {
    println!("MinimizeYTM を起動しました。タスクトレイに常駐します。");
    println!("トレイアイコンをダブルクリックすると YouTube Music の表示/非表示を切り替えます。");

    if let Err(error) = run_tray() {
        println!("トレイ処理でエラーが発生しました: {}", error);
    }
}
