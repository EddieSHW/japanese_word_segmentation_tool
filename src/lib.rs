use lindera::{DictionaryConfig, DictionaryKind, Mode, Tokenizer, TokenizerConfig};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};

/// 形態素解析結果を格納する構造体
#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub text: String, // 単語
    pub pos: String,  // 品詞
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
        let dictionary = DictionaryConfig {
            kind: Some(DictionaryKind::IPADIC),
            path: None,
        };

        let config = TokenizerConfig {
            dictionary,
            user_dictionary: None,
            mode: Mode::Normal,
        };

        // Tokenizerの初期化
        let tokenizer = Tokenizer::from_config(config)?;

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
}
