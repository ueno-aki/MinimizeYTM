# MinimizeYTM

YouTube Music を最小化してタスクトレイに常駐させるシンプルなアプリケーションです。

## 概要

MinimizeYTM は Windows のタスクトレイに常駐し、YouTube Music のウィンドウを最小化・復元できるツールです。Rust で開発されており、軽量で高速に動作します。

## 機能

- **自動最小化**: アプリケーション実行時に YouTube Music ウィンドウを自動的に最小化し、タスクバーからも消える
- **トレイアイコン表示**: YouTube Music のウィンドウアイコンをタスクトレイに表示
- **ダブルクリック復元**: トレイアイコンをダブルクリックすると復元
- **自動終了**: 復元後に自動的にアプリケーションを終了

## 使い方

### 実行方法

```bash
cargo run
```

または、リリースビルドを実行：

```bash
cargo run --release
```

実行すると以下の動作が行われます：

1. YouTube Music のウィンドウが検出され、最小化されます
2. タスクトレイに YouTube Music のアイコンが表示されます
3. トレイアイコンをダブルクリックするとウィンドウが復元され、アプリケーションが終了します

## ビルド

### 開発用ビルド

```bash
cargo build
```

実行ファイルは `target/debug/MinimizeYTM.exe` に生成されます。

### リリース用ビルド

```bash
cargo build --release
```

最適化されたバイナリが `target/release/MinimizeYTM.exe` に生成されます。

実行ファイルは `target/release/MinimizeYTM.exe` から直接実行できます。パスを通しておくと、どのディレクトリからでも `MinimizeYTM.exe` で実行できて便利です。

## 環境要件

- Windows 10 以降
- YouTube Music アプリがインストールされていること
- Rust 開発環境（ビルド時）

## ビルドに必要なツール

- [Rust](https://www.rust-lang.org/) (1.70.0 以降推奨)

## その他のコマンド

```bash
# コード フォーマット
cargo fmt

# Lint チェック
cargo clippy

# テスト実行
cargo test
```

## トラブルシューティング

### YouTube Music のウィンドウが見つからない

アプリケーション実行時に「YouTube Music のウィンドウが見つかりませんでした」というメッセージが表示される場合、YouTube Music が起動していることを確認してください。

### トレイアイコンが表示されない

Windows のタスクトレイ設定で、通知領域に表示するアプリケーションを確認してください。

## ライセンス

このプロジェクトのライセンスについては、LICENSE ファイルを参照してください。

## 開発者向け

### プロジェクト構成

- `src/main.rs` - メインエントリーポイント
- `src/tray.rs` - タスクトレイ操作関連
- `src/window.rs` - ウィンドウ操作関連

### コーディング規約

詳細は `.github/copilot-instructions.md` を参照してください。
