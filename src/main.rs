use eframe::egui;
use jp_word_segment::{
    ConcordanceResult, CooccurrenceConfig, CooccurrenceNetwork, TokenizerCore,
};
use rfd::FileDialog;
use std::collections::HashSet;

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
    network_view: NetworkView,
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
            network_view: NetworkView::default(),
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

            // 共起ネットワークセクション
            ui.collapsing("共起ネットワーク", |ui| {
                self.network_view.ui(ui, &self.core);
                // セクションが開いている時のみ再描画を要求
                if self.network_view.simulating && self.network_view.network.is_some() {
                    ui.ctx().request_repaint();
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

/// 共起ネットワークの表示・操作を行うビュー
struct NetworkView {
    network: Option<CooccurrenceNetwork>,
    positions: Vec<egui::Vec2>,    // ワールド座標 (中心0,0)
    velocities: Vec<egui::Vec2>,
    /// 設定 UI 用フィールド
    pos_nouns: bool,
    pos_verbs: bool,
    pos_adjectives: bool,
    pos_adverbs: bool,
    min_word_freq: usize,
    min_edge_count: usize,
    max_nodes: usize,
    stopwords_input: String,
    /// 表示状態
    simulating: bool,
    zoom: f32,
    pan: egui::Vec2,
    dragging: Option<usize>,
    drag_offset: egui::Vec2,
    selected: Option<usize>,
}

impl Default for NetworkView {
    fn default() -> Self {
        Self {
            network: None,
            positions: Vec::new(),
            velocities: Vec::new(),
            pos_nouns: true,
            pos_verbs: true,
            pos_adjectives: true,
            pos_adverbs: false,
            min_word_freq: 2,
            min_edge_count: 2,
            max_nodes: 60,
            stopwords_input: String::new(),
            simulating: true,
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            dragging: None,
            drag_offset: egui::Vec2::ZERO,
            selected: None,
        }
    }
}

impl NetworkView {
    fn build_config(&self) -> CooccurrenceConfig {
        let mut target_pos = Vec::new();
        if self.pos_nouns {
            target_pos.push("名詞".to_string());
        }
        if self.pos_verbs {
            target_pos.push("動詞".to_string());
        }
        if self.pos_adjectives {
            target_pos.push("形容詞".to_string());
        }
        if self.pos_adverbs {
            target_pos.push("副詞".to_string());
        }

        let stopwords: HashSet<String> = self
            .stopwords_input
            .split(|c: char| c == ',' || c == '、' || c.is_whitespace())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        CooccurrenceConfig {
            target_pos,
            min_word_freq: self.min_word_freq.max(1),
            min_edge_count: self.min_edge_count.max(1),
            max_nodes: self.max_nodes.max(1),
            stopwords,
        }
    }

    fn rebuild(&mut self, core: &TokenizerCore) {
        let config = self.build_config();
        let network = core.compute_cooccurrence(&config);

        // 円周上に初期配置
        let n = network.nodes.len();
        self.positions = (0..n)
            .map(|i| {
                if n == 0 {
                    egui::Vec2::ZERO
                } else {
                    let theta = i as f32 / n as f32 * std::f32::consts::TAU;
                    let r = 200.0;
                    egui::vec2(r * theta.cos(), r * theta.sin())
                }
            })
            .collect();
        self.velocities = vec![egui::Vec2::ZERO; n];
        self.network = Some(network);
        self.simulating = true;
        self.dragging = None;
        self.drag_offset = egui::Vec2::ZERO;
        self.selected = None;
        self.pan = egui::Vec2::ZERO;
        self.zoom = 1.0;
    }

    fn ui(&mut self, ui: &mut egui::Ui, core: &TokenizerCore) {
        // 設定 UI
        ui.horizontal_wrapped(|ui| {
            ui.label("対象品詞:");
            ui.checkbox(&mut self.pos_nouns, "名詞");
            ui.checkbox(&mut self.pos_verbs, "動詞");
            ui.checkbox(&mut self.pos_adjectives, "形容詞");
            ui.checkbox(&mut self.pos_adverbs, "副詞");
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("最小単語頻度:");
            ui.add(
                egui::DragValue::new(&mut self.min_word_freq)
                    .speed(1.0)
                    .clamp_range(1..=100),
            );
            ui.label("最小共起回数:");
            ui.add(
                egui::DragValue::new(&mut self.min_edge_count)
                    .speed(1.0)
                    .clamp_range(1..=100),
            );
            ui.label("最大ノード数:");
            ui.add(
                egui::DragValue::new(&mut self.max_nodes)
                    .speed(1.0)
                    .clamp_range(2..=500),
            );
        });
        ui.horizontal(|ui| {
            ui.label("除外単語 (カンマ/空白区切り):");
            ui.text_edit_singleline(&mut self.stopwords_input);
        });

        ui.horizontal(|ui| {
            if ui.button("ネットワークを計算").clicked() {
                self.rebuild(core);
            }
            if self.network.is_some() {
                let label = if self.simulating {
                    "レイアウト停止"
                } else {
                    "レイアウト再開"
                };
                if ui.button(label).clicked() {
                    self.simulating = !self.simulating;
                }
                if ui.button("配置リセット").clicked() {
                    self.rebuild(core);
                }
            }
        });

        if let Some(network) = &self.network {
            ui.label(format!(
                "ノード: {} 件 / エッジ: {} 件",
                network.nodes.len(),
                network.edges.len()
            ));
        }

        if self.network.is_some() {
            self.draw_canvas(ui);
        } else {
            ui.label("「ネットワークを計算」ボタンを押すと共起ネットワークが表示されます。");
        }
    }

    fn draw_canvas(&mut self, ui: &mut egui::Ui) {
        let desired_size = egui::vec2(ui.available_width(), 480.0);
        let (response, painter) = ui.allocate_painter(desired_size, egui::Sense::click_and_drag());
        let rect = response.rect;
        let center = rect.center();

        // 背景
        painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(20, 22, 28));
        painter.rect_stroke(
            rect,
            4.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 70)),
        );

        // ズーム (ホバー時のみ)
        if response.hovered() {
            let scroll = ui.ctx().input(|i| i.scroll_delta.y);
            if scroll != 0.0 {
                let factor = (scroll * 0.0015).exp();
                self.zoom = (self.zoom * factor).clamp(0.2, 5.0);
            }
        }

        // ローカル変数にスナップショットしてクロージャで使用 (ボローチェッカー対策)
        let mut pan = self.pan;
        let zoom = self.zoom;
        let to_screen = |w: egui::Vec2, pan: egui::Vec2| -> egui::Pos2 {
            center + (w + pan) * zoom
        };
        let to_world = |s: egui::Pos2, pan: egui::Vec2| -> egui::Vec2 {
            (s - center) / zoom - pan
        };

        // ヒットテスト + ドラッグ処理
        let pointer_screen = response.interact_pointer_pos();
        let hover_screen = response.hover_pos();

        if response.drag_started() {
            if let Some(p) = pointer_screen {
                let target = to_world(p, pan);
                let mut hit: Option<usize> = None;
                if let Some(network) = &self.network {
                    for (i, pos) in self.positions.iter().enumerate() {
                        let r = node_radius(network.nodes[i].frequency);
                        // ワールド距離 r、加えて画面 4px 分の余白
                        if (*pos - target).length() <= r + 4.0 / zoom {
                            hit = Some(i);
                            break;
                        }
                    }
                }
                self.dragging = hit;
                // 押下位置とノード中心のオフセットを保持しテレポートを防ぐ
                self.drag_offset = match hit {
                    Some(idx) => self.positions[idx] - target,
                    None => egui::Vec2::ZERO,
                };
                self.selected = hit.or(self.selected);
            }
        }

        if response.dragged() {
            if let (Some(idx), Some(p)) = (self.dragging, pointer_screen) {
                self.positions[idx] = to_world(p, pan) + self.drag_offset;
                if idx < self.velocities.len() {
                    self.velocities[idx] = egui::Vec2::ZERO;
                }
            } else {
                // 空白部分のドラッグ → パン
                let delta = response.drag_delta();
                pan += delta / zoom;
            }
        }

        if response.drag_released() {
            self.dragging = None;
            self.drag_offset = egui::Vec2::ZERO;
        }

        // クリック (移動なし) で選択クリア/トグル
        if response.clicked() {
            // interact_pointer_pos は press 位置を返すので click 判定にも妥当
            if let Some(p) = pointer_screen.or(hover_screen) {
                let target = to_world(p, pan);
                let mut hit: Option<usize> = None;
                if let Some(network) = &self.network {
                    for (i, pos) in self.positions.iter().enumerate() {
                        let r = node_radius(network.nodes[i].frequency);
                        if (*pos - target).length() <= r + 4.0 / zoom {
                            hit = Some(i);
                            break;
                        }
                    }
                }
                self.selected = hit;
            }
        }

        // 更新したパンを書き戻す
        self.pan = pan;

        // 物理シミュレーション 1 ステップ
        if self.simulating {
            self.step_layout(rect.size());
        }

        // 描画: エッジ
        if let Some(network) = &self.network {
            let max_edge = network.edges.iter().map(|e| e.count).max().unwrap_or(1) as f32;
            let highlight = self.selected;

            // 弱いエッジを先に描画して強いエッジが上に重なるようにする
            for edge in network.edges.iter().rev() {
                let a = to_screen(self.positions[edge.source], pan);
                let b = to_screen(self.positions[edge.target], pan);
                let weight = (edge.count as f32) / max_edge;
                let width = (0.6 + weight * 3.5).min(5.0);
                let is_highlight = match highlight {
                    Some(i) => edge.source == i || edge.target == i,
                    None => false,
                };
                let color = if is_highlight {
                    egui::Color32::from_rgb(240, 180, 80)
                } else {
                    let v = (60.0 + weight * 120.0) as u8;
                    egui::Color32::from_rgba_unmultiplied(v, v, v.saturating_add(20), 200)
                };
                painter.line_segment([a, b], egui::Stroke::new(width, color));
            }

            // 描画: ノード
            let max_freq = network
                .nodes
                .iter()
                .map(|n| n.frequency)
                .max()
                .unwrap_or(1) as f32;
            for (i, node) in network.nodes.iter().enumerate() {
                let p = to_screen(self.positions[i], pan);
                let r = node_radius(node.frequency) * zoom;
                let intensity = (node.frequency as f32 / max_freq).sqrt();
                let fill = lerp_color(
                    egui::Color32::from_rgb(80, 130, 220),
                    egui::Color32::from_rgb(230, 90, 130),
                    intensity,
                );
                let stroke = if Some(i) == highlight {
                    egui::Stroke::new(2.5, egui::Color32::from_rgb(255, 220, 120))
                } else {
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(20, 20, 30))
                };
                painter.circle(p, r, fill, stroke);

                // ラベル
                let font_size = (10.0 + intensity * 6.0).min(20.0);
                painter.text(
                    p + egui::vec2(0.0, -r - 2.0),
                    egui::Align2::CENTER_BOTTOM,
                    &node.word,
                    egui::FontId::proportional(font_size),
                    egui::Color32::from_rgb(235, 235, 240),
                );
            }
        }

        // 凡例
        painter.text(
            rect.left_top() + egui::vec2(8.0, 6.0),
            egui::Align2::LEFT_TOP,
            "ドラッグ: ノード移動 / 空白ドラッグ: パン / ホイール: ズーム",
            egui::FontId::proportional(11.0),
            egui::Color32::from_rgb(160, 160, 170),
        );
    }

    fn step_layout(&mut self, area: egui::Vec2) {
        let Some(network) = &self.network else {
            return;
        };
        let n = network.nodes.len();
        if n < 2 {
            return;
        }

        // 最適距離 k
        let k = (area.x * area.y / n as f32).sqrt() * 0.35;
        let dt = 0.6;

        let mut forces = vec![egui::Vec2::ZERO; n];

        // 反発力 O(n^2)
        for i in 0..n {
            for j in (i + 1)..n {
                let mut delta = self.positions[i] - self.positions[j];
                let mut dist = delta.length();
                if dist < 0.01 {
                    delta = egui::vec2(((i as f32) * 0.3).cos(), ((i as f32) * 0.7).sin());
                    dist = 0.01;
                }
                let f = (k * k) / dist;
                let dir = delta / dist;
                forces[i] += dir * f;
                forces[j] -= dir * f;
            }
        }

        // 引力 (エッジに沿って)
        let max_count = network.edges.iter().map(|e| e.count).max().unwrap_or(1) as f32;
        for edge in &network.edges {
            let delta = self.positions[edge.target] - self.positions[edge.source];
            let dist = delta.length().max(0.01);
            let weight = 0.5 + 0.5 * (edge.count as f32 / max_count);
            let f = (dist * dist) / k * weight;
            let dir = delta / dist;
            forces[edge.source] += dir * f;
            forces[edge.target] -= dir * f;
        }

        // 中心への弱い引力
        for i in 0..n {
            forces[i] += -self.positions[i] * 0.01;
        }

        // 速度・位置更新 (温度減衰付き)
        let max_speed = 60.0;
        for i in 0..n {
            if Some(i) == self.dragging {
                self.velocities[i] = egui::Vec2::ZERO;
                continue;
            }
            self.velocities[i] = (self.velocities[i] + forces[i] * dt) * 0.82;
            let speed = self.velocities[i].length();
            if speed > max_speed {
                self.velocities[i] *= max_speed / speed;
            }
            self.positions[i] += self.velocities[i] * dt;
        }
    }
}

/// ノードの頻度から半径を決定 (ワールド座標)
fn node_radius(freq: usize) -> f32 {
    6.0 + (freq as f32).ln_1p() * 4.5
}

/// 2 色を線形補間する
fn lerp_color(a: egui::Color32, b: egui::Color32, t: f32) -> egui::Color32 {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: u8, y: u8| -> u8 {
        (x as f32 * (1.0 - t) + y as f32 * t).round().clamp(0.0, 255.0) as u8
    };
    egui::Color32::from_rgb(
        lerp(a.r(), b.r()),
        lerp(a.g(), b.g()),
        lerp(a.b(), b.b()),
    )
}
