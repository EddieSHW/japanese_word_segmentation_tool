use lindera::tokenizer::{Tokenizer, TokenizerConfig};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Read, Write};

/// 形態素解析結果を格納する構造体
#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub text: String, // 単語
    pub pos: String,  // 品詞
}

/// コンコーダンス結果を格納する構造体
#[derive(Debug, Clone)]
pub struct ConcordanceResult {
    pub keyword: String,       // 検索キーワード
    pub left_context: String,  // 左文脈
    pub right_context: String, // 右文脈
    pub line_number: usize,    // 行番号
}

/// 共起ネットワークの計算条件
#[derive(Debug, Clone)]
pub struct CooccurrenceConfig {
    pub target_pos: Vec<String>,    // 対象品詞 (例: ["名詞", "動詞", "形容詞"])
    pub min_word_freq: usize,       // 最小単語頻度
    pub min_edge_count: usize,      // 最小共起回数
    pub max_nodes: usize,           // 最大ノード数 (頻度上位)
    pub stopwords: HashSet<String>, // 除外単語
}

impl Default for CooccurrenceConfig {
    fn default() -> Self {
        Self {
            target_pos: vec!["名詞".to_string(), "動詞".to_string(), "形容詞".to_string()],
            min_word_freq: 2,
            min_edge_count: 2,
            max_nodes: 60,
            stopwords: HashSet::new(),
        }
    }
}

/// 共起ネットワークのノード
#[derive(Debug, Clone)]
pub struct CooccurrenceNode {
    pub word: String,
    pub frequency: usize,
}

/// 共起ネットワークのエッジ
#[derive(Debug, Clone)]
pub struct CooccurrenceEdge {
    pub source: usize, // ノードインデックス
    pub target: usize, // ノードインデックス
    pub count: usize,  // 共起回数
}

/// 共起ネットワーク全体
#[derive(Debug, Clone, Default)]
pub struct CooccurrenceNetwork {
    pub nodes: Vec<CooccurrenceNode>,
    pub edges: Vec<CooccurrenceEdge>,
}

/// 入力テキストを文単位に分割する (。!?。改行 を区切りとして使用)
fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        match ch {
            '。' | '．' | '!' | '！' | '?' | '？' | '\n' | '\r' => {
                if !current.trim().is_empty() {
                    sentences.push(std::mem::take(&mut current));
                } else {
                    current.clear();
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.trim().is_empty() {
        sentences.push(current);
    }
    sentences
}

/// アプリケーションのメインロジックを管理する構造体
pub struct TokenizerCore {
    pub tokenizer: Tokenizer,
    pub input_text: String,
    pub tokens: Vec<TokenInfo>,
    pub word_frequencies: HashMap<String, usize>,
    pub file_path: Option<String>,
}

impl TokenizerCore {
    /// 新しいTokenizerCoreインスタンスを作成
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config: TokenizerConfig = json!({
            "segmenter": {
                "dictionary": "embedded://ipadic",
                "mode": "normal"
            },
            "character_filters": [],
            "token_filters": []
        });

        // Tokenizerの初期化
        let tokenizer = Tokenizer::from_config(&config)?;

        Ok(Self {
            tokenizer,
            input_text: String::new(),
            tokens: Vec::new(),
            word_frequencies: HashMap::new(),
            file_path: None,
        })
    }

    /// テキストを形態素解析する
    pub fn analyze_text(&mut self) {
        self.tokens.clear();
        self.word_frequencies.clear();

        if let Ok(tokens) = self.tokenizer.tokenize(&self.input_text) {
            for mut token in tokens {
                let text = token.surface.to_string();
                let pos = token.get_detail(0).unwrap_or("*").to_string();

                self.tokens.push(TokenInfo {
                    text: text.clone(),
                    pos,
                });

                *self.word_frequencies.entry(text).or_insert(0) += 1;
            }
        }
    }

    /// ファイルからテキストを読み込む
    pub fn load_file(&mut self, path: std::path::PathBuf) -> Result<(), std::io::Error> {
        let mut file = File::open(&path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        self.input_text = content;
        self.file_path = Some(path.to_string_lossy().into_owned());
        Ok(())
    }

    /// 解析結果をCSVファイルに保存
    pub fn save_to_csv(&self, path: std::path::PathBuf) -> Result<(), std::io::Error> {
        if self.tokens.is_empty() {
            return Ok(());
        }

        let mut file = File::create(path)?;

        // UTF-8 BOMを書き込み（Excel対応）
        file.write_all(&[0xEF, 0xBB, 0xBF])?;

        // ヘッダーを書き込み
        writeln!(file, "単語,品詞,出現頻度")?;

        // データを書き込み
        for info in &self.tokens {
            let frequency = self.word_frequencies.get(&info.text).unwrap_or(&0);
            writeln!(file, "{},{},{}", info.text, info.pos, frequency)?;
        }

        Ok(())
    }

    /// 共起ネットワークを計算する
    ///
    /// 文単位 (。!? および改行で区切る) で同一文に出現する単語ペアの共起回数を集計する。
    /// 同じ文内で同一単語ペアが複数回出現しても 1 回としてカウントする。
    pub fn compute_cooccurrence(&self, config: &CooccurrenceConfig) -> CooccurrenceNetwork {
        let sentences = split_sentences(&self.input_text);

        let mut sentence_words: Vec<HashSet<String>> = Vec::with_capacity(sentences.len());
        let mut word_freq: HashMap<String, usize> = HashMap::new();

        for sentence in &sentences {
            let mut words_in_sentence: HashSet<String> = HashSet::new();
            if let Ok(tokens) = self.tokenizer.tokenize(sentence) {
                for mut token in tokens {
                    let pos = token.get_detail(0).unwrap_or("*").to_string();
                    if !config.target_pos.iter().any(|p| p == &pos) {
                        continue;
                    }
                    let word = token.surface.to_string();
                    if word.trim().is_empty() {
                        continue;
                    }
                    if config.stopwords.contains(&word) {
                        continue;
                    }
                    *word_freq.entry(word.clone()).or_insert(0) += 1;
                    words_in_sentence.insert(word);
                }
            }
            sentence_words.push(words_in_sentence);
        }

        let mut sorted: Vec<(String, usize)> = word_freq
            .into_iter()
            .filter(|(_, c)| *c >= config.min_word_freq)
            .collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        sorted.truncate(config.max_nodes);

        let nodes: Vec<CooccurrenceNode> = sorted
            .into_iter()
            .map(|(word, frequency)| CooccurrenceNode { word, frequency })
            .collect();

        let word_to_idx: HashMap<&str, usize> = nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (n.word.as_str(), i))
            .collect();

        let mut edge_counts: HashMap<(usize, usize), usize> = HashMap::new();
        for words in &sentence_words {
            let mut indices: Vec<usize> = words
                .iter()
                .filter_map(|w| word_to_idx.get(w.as_str()).copied())
                .collect();
            indices.sort_unstable();
            indices.dedup();
            for i in 0..indices.len() {
                for j in (i + 1)..indices.len() {
                    let key = (indices[i], indices[j]);
                    *edge_counts.entry(key).or_insert(0) += 1;
                }
            }
        }

        let mut edges: Vec<CooccurrenceEdge> = edge_counts
            .into_iter()
            .filter(|(_, c)| *c >= config.min_edge_count)
            .map(|((s, t), count)| CooccurrenceEdge {
                source: s,
                target: t,
                count,
            })
            .collect();
        edges.sort_by(|a, b| b.count.cmp(&a.count));

        CooccurrenceNetwork { nodes, edges }
    }

    /// コンコーダンス検索を実行
    pub fn search_concordance(&self, keyword: &str, context_size: usize) -> Vec<ConcordanceResult> {
        let mut results = Vec::new();
        let lines: Vec<&str> = self.input_text.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            if let Ok(tokens) = self.tokenizer.tokenize(line) {
                for (i, token) in tokens.iter().enumerate() {
                    if token.surface == keyword {
                        let mut left_context = String::new();
                        let mut right_context = String::new();

                        // 左文脈の取得
                        let start = if i > context_size {
                            i - context_size
                        } else {
                            0
                        };
                        for t in &tokens[start..i] {
                            left_context.push_str(&t.surface);
                        }

                        // 右文脈の取得
                        let end = if i + context_size < tokens.len() {
                            i + context_size
                        } else {
                            tokens.len()
                        };
                        for t in &tokens[i + 1..end] {
                            right_context.push_str(&t.surface);
                        }

                        results.push(ConcordanceResult {
                            keyword: keyword.to_string(),
                            left_context,
                            right_context,
                            line_number: line_num + 1,
                        });
                    }
                }
            }
        }

        results
    }
}
