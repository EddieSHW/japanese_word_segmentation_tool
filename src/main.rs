use eframe::egui;
use lindera::Mode;
use lindera::{DictionaryConfig, DictionaryKind};
use lindera::{Tokenizer, TokenizerConfig};
use rfd::FileDialog;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

fn main() -> Result<(), eframe::Error> {
    let dictionary = DictionaryConfig {
        kind: Some(DictionaryKind::IPADIC),
        path: None,
    };

    let config = TokenizerConfig {
        dictionary,
        user_dictionary: None,
        mode: Mode::Normal,
    };

    // create tokenizer
    let tokenizer = Tokenizer::from_config(config).expect("Failed to create tokenizer");

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "日本語形態素解析アプリ",
        native_options,
        Box::new(|cc| Box::new(MyApp::new(cc, tokenizer))),
    )?;

    Ok(())
}

struct MyApp {
    tokenizer: Tokenizer,
    input_text: String,
    tokens: Vec<TokenInfo>,
    word_frequencies: HashMap<String, usize>,
    file_path: Option<String>,
}

#[derive(Debug, Clone)]
struct TokenInfo {
    text: String,
    pos: String,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>, tokenizer: Tokenizer) -> Self {
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
            tokenizer,
            input_text: String::new(),
            tokens: Vec::new(),
            word_frequencies: HashMap::new(),
            file_path: None,
        }
    }

    fn analyze_text(&mut self) {
        self.tokens.clear();
        self.word_frequencies.clear();

        if let Ok(tokens) = self.tokenizer.tokenize(&self.input_text) {
            for mut token in tokens {
                let text = token.text.to_string();
                let pos = token.get_details().unwrap()[0].to_string();

                self.tokens.push(TokenInfo {
                    text: text.clone(),
                    pos,
                });

                *self.word_frequencies.entry(text).or_insert(0) += 1;
            }
        }
    }

    fn load_file(&mut self) -> Result<(), std::io::Error> {
        if let Some(path) = FileDialog::new().pick_file() {
            let mut file = File::open(&path)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            self.input_text = content;
            self.file_path = Some(path.to_string_lossy().into_owned());
        }
        Ok(())
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("日本語形態素解析");

            if ui.button("ファイルを開く").clicked() {
                if let Err(e) = self.load_file() {
                    eprintln!("ファイル読み込みエラー: {}", e);
                }
            }

            if let Some(path) = &self.file_path {
                ui.label(format!("読み込んだファイル: {}", path));
            }

            ui.text_edit_multiline(&mut self.input_text);
            if ui.button("解析").clicked() {
                self.analyze_text();
            }

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("tokens_grid").striped(true).show(ui, |ui| {
                    ui.heading("単語");
                    ui.heading("品詞");
                    ui.heading("頻度");
                    ui.end_row();

                    for info in &self.tokens {
                        ui.label(&info.text);
                        ui.label(&info.pos);
                        ui.label(
                            self.word_frequencies
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
