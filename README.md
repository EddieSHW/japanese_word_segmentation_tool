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
- 使いやすい GUI インターフェース

## 必要要件

- Rust 1.70.0 以上
- Cargo（Rust のパッケージマネージャー）

## インストール方法

1. このリポジトリをクローン：

```bash
git clone https://github.com/EddieSHW/jp_word_segment.git
cd jp_word_segment
```

2. 依存パッケージのインストール：

```bash
cargo build --release
```

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

## 依存クレート

- [lindera](https://github.com/lindera-morphology/lindera) - 形態素解析エンジン
- [eframe](https://github.com/emilk/egui) - GUI フレームワーク
- [rfd](https://github.com/PolyMeilex/rfd) - ファイルダイアログ

## ライセンス

MIT License

## 注意事項

- 解析結果の CSV ファイルは UTF-8（BOM 付き）で保存されます
- 大きなテキストファイルの処理には時間がかかる場合があります

## 開発者向け情報

プロジェクト構造：

- `src/main.rs` - GUI アプリケーションの実装
- `src/lib.rs` - コアロジック（形態素解析、ファイル操作）
- `Cargo.toml` - 依存関係の管理

## 貢献

1. Fork を作成
2. 新しいブランチを作成 (`git checkout -b feature/amazing-feature`)
3. 変更をコミット (`git commit -m 'Add some amazing feature'`)
4. ブランチにプッシュ (`git push origin feature/amazing-feature`)
5. Pull Request を作成

## 連絡先

問題や提案がありましたら、GitHub の Issue を作成してください。
