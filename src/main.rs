use eframe::egui;
use jp_word_segment::{ConcordanceResult, TokenizerCore};
use rfd::FileDialog;

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "日本語形態素解析アプリ",
        native_options,
        Box::new(|cc| Box::new(TokenizerApp::new(cc))),
    )?;

    Ok(())
}

/// GUIアプリケーションの構造体
struct TokenizerApp {
    core: TokenizerCore,
    search_keyword: String,
    context_size: usize,
    concordance_results: Vec<ConcordanceResult>,
    show_concordance: bool,
}

impl TokenizerApp {
    /// 新しいTokenizerAppインスタンスを作成
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // フォントの設定
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "noto_sans_jp".to_owned(),
            egui::FontData::from_static(include_bytes!(
                "../assets/Noto_Sans_JP/static/NotoSansJP-Regular.ttf"
            )),
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "noto_sans_jp".to_owned());

        cc.egui_ctx.set_fonts(fonts);

        Self {
            core: TokenizerCore::new().expect("Failed to initialize TokenizerCore"),
            search_keyword: String::new(),
            context_size: 5,
            concordance_results: Vec::new(),
            show_concordance: false,
        }
    }
}

impl eframe::App for TokenizerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("日本語形態素解析");

            // ファイル読み込みボタン
            if ui.button("ファイルを開く").clicked() {
                if let Some(path) = FileDialog::new().pick_file() {
                    if let Err(e) = self.core.load_file(path) {
                        eprintln!("ファイル読み込みエラー: {}", e);
                    }
                }
            }

            // 読み込んだファイルパスの表示
            if let Some(path) = &self.core.file_path {
                ui.label(format!("読み込んだファイル: {}", path));
            }

            // テキスト入力エリア
            ui.text_edit_multiline(&mut self.core.input_text);

            // 解析ボタン
            if ui.button("解析").clicked() {
                self.core.analyze_text();
            }

            // CSV保存ボタン
            if !self.core.tokens.is_empty() {
                if ui.button("CSVファイルに保存").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("CSV", &["csv"])
                        .set_file_name("morphological_analysis.csv")
                        .save_file()
                    {
                        if let Err(e) = self.core.save_to_csv(path) {
                            eprintln!("CSV保存エラー: {}", e);
                        }
                    }
                }
            }

            ui.separator();

            // コンコーダンス検索セクション
            ui.collapsing("コンコーダンス検索", |ui| {
                ui.horizontal(|ui| {
                    ui.label("検索キーワード:");
                    ui.text_edit_singleline(&mut self.search_keyword);
                    ui.label("文脈サイズ:");
                    ui.add(
                        egui::DragValue::new(&mut self.context_size)
                            .speed(1.0)
                            .clamp_range(1..=20),
                    );
                });

                if ui.button("検索").clicked() && !self.search_keyword.is_empty() {
                    self.concordance_results = self
                        .core
                        .search_concordance(&self.search_keyword, self.context_size);
                    self.show_concordance = true;
                }

                if self.show_concordance {
                    ui.label(format!("検索結果: {}件", self.concordance_results.len()));

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::Grid::new("concordance_grid")
                            .striped(true)
                            .show(ui, |ui| {
                                ui.heading("行番号");
                                ui.heading("左文脈");
                                ui.heading("キーワード");
                                ui.heading("右文脈");
                                ui.end_row();

                                for result in &self.concordance_results {
                                    ui.label(result.line_number.to_string());
                                    ui.label(&result.left_context);
                                    ui.label(&result.keyword);
                                    ui.label(&result.right_context);
                                    ui.end_row();
                                }
                            });
                    });
                }
            });

            ui.separator();

            // 解析結果の表示
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("tokens_grid").striped(true).show(ui, |ui| {
                    ui.heading("単語");
                    ui.heading("品詞");
                    ui.heading("頻度");
                    ui.end_row();

                    for info in &self.core.tokens {
                        ui.label(&info.text);
                        ui.label(&info.pos);
                        ui.label(
                            self.core
                                .word_frequencies
                                .get(&info.text)
                                .unwrap_or(&0)
                                .to_string(),
                        );
                        ui.end_row();
                    }
                });
            });
        });
    }
}
