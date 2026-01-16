# diff-tui

ターミナル上で動作するGit差分表示ツールです。変更ファイルの選択とdiffの閲覧を直感的に行えます。

## 機能

- **ファイル選択画面**: Gitリポジトリ内の変更ファイル一覧を表示
- **ファジー検索**: `/`キーで検索モードに入り、ファイル名を絞り込み
- **差分表示**: deltaまたはgit diffによる色付き差分表示
- **ステータス表示**: M(変更), A(追加), D(削除), R(リネーム), ?(未追跡)

## 必要要件

- Rust 1.70以上
- Git

### オプション

- [delta](https://github.com/dandavison/delta) - より見やすい差分表示（未インストールの場合はgit diffにフォールバック）

## インストール

```bash
cargo install --path .
```

または開発用:

```bash
cargo build --release
./target/release/diff-tui
```

## 設定

`~/.config/diff-tui/config.toml` を作成して設定をカスタマイズできます:

```toml
[diff]
# 使用するdiffツール（デフォルト: "auto"）
# 選択肢: "auto", "delta", "diff-so-fancy", "difftastic", "colordiff", "git"
# または任意のコマンド名を指定可能
tool = "delta"

# diffツールに渡す追加引数（オプション）
args = ["--side-by-side"]
```

### Diffツールの動作

| 値 | 動作 |
|----|------|
| `"auto"` | deltaを試し、なければgit diffにフォールバック（デフォルト） |
| `"delta"` | deltaを使用（未インストールの場合はgit diffにフォールバック） |
| `"git"` | git diffを直接使用 |
| その他 | 指定したコマンドを使用（見つからない場合はgit diffにフォールバック） |

## 使い方

Gitリポジトリ内で実行:

```bash
diff-tui
```

### キーバインド

#### ファイル選択画面

| キー | アクション |
|------|------------|
| `j` / `↓` | 次のファイルへ移動 |
| `k` / `↑` | 前のファイルへ移動 |
| `Enter` | 選択したファイルの差分を表示 |
| `/` | 検索モードを開始 |
| `q` | 終了 |

#### 検索モード

| キー | アクション |
|------|------------|
| 文字入力 | 検索クエリを追加 |
| `Backspace` | 1文字削除 |
| `Enter` | 選択したファイルの差分を表示 |
| `Esc` | 検索をキャンセル |
| `↑` / `↓` | 検索結果内で移動 |

#### 差分表示画面

| キー | アクション |
|------|------------|
| `j` / `↓` | 1行スクロールダウン |
| `k` / `↑` | 1行スクロールアップ |
| `d` / `PageDown` | 20行スクロールダウン |
| `u` / `PageUp` | 20行スクロールアップ |
| `g` / `Home` | 先頭へ移動 |
| `G` / `End` | 末尾へ移動 |
| `q` / `Esc` | ファイル選択画面に戻る |

## 技術スタック

- [Ratatui](https://ratatui.rs/) - TUIフレームワーク
- [git2](https://crates.io/crates/git2) - Git操作
- [nucleo](https://crates.io/crates/nucleo) - ファジー検索
- [ansi-to-tui](https://crates.io/crates/ansi-to-tui) - ANSIエスケープシーケンスの解析

## ライセンス

MIT
