use ratatui::style::Color;
use tree_sitter_highlight::HighlightConfiguration;

pub const HIGHLIGHTED_TOKENS: &[&str] = &["keyword", "string"];

pub const HIGHLIGHT_THEME: &[(&str, Color)] =
    &[("keyword", Color::Yellow), ("string", Color::Green)];

pub fn get_highlight_color(token_type: &str) -> Option<Color> {
    for (k, c) in HIGHLIGHT_THEME.iter() {
        if *k == token_type {
            return Some(*c);
        }
    }
    None
}

pub enum Language {
    Rust,
}

impl Language {
    pub fn by_file_name(s: &str) -> Option<Language> {
        if s.ends_with(".rs") {
            Some(Language::Rust)
        } else {
            None
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
        }
    }

    pub fn build_highlighter_config(&self) -> Option<HighlightConfiguration> {
        match self {
            Language::Rust => {
                let rust_language = tree_sitter_rust::language();
                let mut rust_config = HighlightConfiguration::new(
                    rust_language,
                    tree_sitter_rust::HIGHLIGHT_QUERY,
                    tree_sitter_rust::INJECTIONS_QUERY,
                    "",
                )
                .ok()?;
                rust_config.configure(HIGHLIGHTED_TOKENS);
                Some(rust_config)
            }
            _ => None,
        }
    }
}
