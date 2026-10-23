# 日本語形態素解析アプリ (Japanese Word Segmentation Tool)

## 概要

このアプリケーションは、日本語テキストの形態素解析を行い、単語の分割と品詞の特定を行う GUI ツールです。テキストファイルの読み込み、解析結果の CSV エクスポート、コンコーダンス検索（KWIC）などの機能を提供します。

## 主な機能

- 日本語テキストの形態素解析
- テキストファイルの読み込み
- 解析結果の表示（単語、品詞、出現頻度）
- 解析結果の CSV ファイルエクスポート（Excel 対応）
- コンコーダンス検索（KWIC 形式）
  - 検索語の前後の文脈を表示
  - 文脈サイズの調整機能（1-20 単語）
  - 行番号表示による原文参照
- 共起ネットワークの可視化
  - 文単位（。!?・改行で分割）で同一文に出現する単語の共起関係をグラフ表示
  - 対象品詞（名詞 / 動詞 / 形容詞 / 副詞）を切り替え可能
  - 最小単語頻度・最小共起回数・最大ノード数・除外単語の設定
  - 力指向レイアウトによる自動配置（収束すると自動停止。ノードドラッグ・パン・ホイールズーム対応）
  - ノードクリックで関連エッジをハイライト
- 使いやすい GUI インターフェース

## 必要要件

- Rust 1.88.0 以上（依存する lindera 2.x の要件）
- Cargo（Rust のパッケージマネージャー）
- Linux の場合は GTK3 の開発パッケージ（ファイルダイアログで使用）
  - Debian / Ubuntu: `sudo apt install libgtk-3-dev`

## インストール方法

1. このリポジトリをクローン：

```bash
git clone https://github.com/EddieSHW/japanese_word_segmentation_tool.git
cd japanese_word_segmentation_tool
```

2. 依存パッケージのインストール：

```bash
cargo build --release
```

初回ビルド時に IPADIC 辞書をダウンロードしてバイナリに埋め込むため、インターネット接続が必要です。

## 使用方法

1. アプリケーションの起動：

```bash
cargo run --release
```

2. GUI ウィンドウが開きます。以下の操作が可能です：
   - 「ファイルを開く」ボタンでテキストファイルを読み込み
   - テキスト入力エリアに直接テキストを入力
   - 「解析」ボタンで形態素解析を実行
   - 「CSV ファイルに保存」ボタンで解析結果をエクスポート
   - 「コンコーダンス検索」セクションで特定の単語の用例を検索
     - 検索キーワードを入力
     - 文脈サイズを調整（1-20 単語）
     - 「検索」ボタンで結果を表示
   - 「共起ネットワーク」セクションで単語の共起関係を可視化
     - 対象品詞・最小頻度・最大ノード数・除外単語を設定
     - 「ネットワークを計算」ボタンで描画開始
     - レイアウトの停止・配置リセットが可能
     - ノードはドラッグで固定、空白部分のドラッグでパン、ホイールで拡大縮小

## 依存クレート

- [lindera](https://github.com/lindera-morphology/lindera) - 形態素解析エンジン
- [eframe](https://github.com/emilk/egui) - GUI フレームワーク
- [egui_extras](https://github.com/emilk/egui) - 結果表示用のテーブル
- [rfd](https://github.com/PolyMeilex/rfd) - ファイルダイアログ

## ライセンス

MIT License

## 注意事項

- 読み込むテキストファイルは UTF-8（BOM 付きも可）にしてください。Shift_JIS などのファイルは UTF-8 に変換してから読み込んでください
- 解析結果の CSV ファイルは UTF-8（BOM 付き）で保存されます。カンマやダブルクォートを含む単語は RFC 4180 に従ってクォートされます
- 大きなテキストファイルの処理には時間がかかる場合があります

## 開発者向け情報

プロジェクト構造：

- `src/main.rs` - GUI アプリケーションの実装
- `src/lib.rs` - コアロジック（形態素解析、ファイル操作）とユニットテスト
- `tests/santouka.txt` - 動作確認用のサンプルテキスト
- `Cargo.toml` - 依存関係の管理

テストの実行：

```bash
cargo test
```

## 貢献

1. Fork を作成
2. 新しいブランチを作成 (`git checkout -b feature/amazing-feature`)
3. 変更をコミット (`git commit -m 'Add some amazing feature'`)
4. ブランチにプッシュ (`git push origin feature/amazing-feature`)
5. Pull Request を作成

## 連絡先

問題や提案がありましたら、GitHub の Issue を作成してください。
